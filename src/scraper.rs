use anyhow::{Context, Result};
use headless_chrome::Browser;

// Embed Readability.js at compile time
const READABILITY_JS: &str = include_str!("../scripts/Readability.js");

pub struct Scraper;

impl Scraper {
    pub fn scrape_url(browser: Browser, url: &str) -> Result<(String, String)> {
        let tab = browser.new_tab().context("Failed to create new tab")?;

        tab.navigate_to(url).context("Failed to navigate")?;
        tab.wait_until_navigated().context("Failed to wait for navigation")?;
        std::thread::sleep(std::time::Duration::from_millis(1000));

        let title = tab.get_title().unwrap_or_default();
        let html = tab.evaluate("document.documentElement.outerHTML", false)?
            .value
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_default();

        let markdown = html2md::parse_html(&html);
        Ok((markdown, title))
    }

    pub fn scrape_readable_url(browser: Browser, url: &str) -> Result<(String, String, String)> {
        let tab = browser.new_tab().context("Failed to create new tab")?;

        tab.navigate_to(url).context("Failed to navigate")?;
        tab.wait_until_navigated().context("Failed to wait for navigation")?;
        std::thread::sleep(std::time::Duration::from_millis(1500));

        // 1. Scroll to trigger lazy loading
        let scroll_js = r#"
            (async () => {
                for (let i = 0; i < document.body.scrollHeight; i += 500) {
                    window.scrollTo(0, i);
                    await new Promise(r => setTimeout(r, 100));
                }
                window.scrollTo(0, 0);
            })()
        "#;
        let _ = tab.evaluate(scroll_js, true);
        std::thread::sleep(std::time::Duration::from_millis(500));

        // 2. Fix lazy-loaded images (same as scrapper)
        let fix_images_js = r#"
            document.querySelectorAll('img').forEach(img => {
                const realSrc = img.getAttribute('data-src')
                    || img.getAttribute('data-lazy-src')
                    || img.getAttribute('data-original')
                    || img.getAttribute('data-lazy')
                    || img.src;
                if (realSrc && realSrc.length > 10 && !realSrc.startsWith('data:')) {
                    img.src = realSrc;
                    img.setAttribute('src', realSrc);
                }
                img.removeAttribute('srcset');
                img.removeAttribute('data-srcset');
            });
        "#;
        tab.evaluate(fix_images_js, false)?;

        // 3. Inject Readability.js
        tab.evaluate(READABILITY_JS, false)
            .context("Failed to inject Readability.js")?;

        // 4. Remove hidden elements + Run Readability (same as scrapper's article.js)
        let article_js = r#"
            (function() {
                try {
                    // Remove hidden elements (from scrapper's article.js)
                    function isHidden(el) {
                        if (!el || el.nodeType !== 1) return false;
                        const style = window.getComputedStyle(el);
                        if (style.display === 'none') return true;
                        if (style.visibility === 'hidden') return true;
                        if (parseFloat(style.opacity) === 0) return true;
                        const rect = el.getBoundingClientRect();
                        if (rect.width <= 1 && rect.height <= 1) return true;
                        return false;
                    }

                    // Mark and remove hidden elements
                    document.querySelectorAll('*').forEach(el => {
                        if (isHidden(el) && !el.querySelector('img')) {
                            el.classList.add('scrapper-hidden');
                        }
                    });
                    document.querySelectorAll('.scrapper-hidden').forEach(el => el.remove());

                    // Remove comments
                    function removeComments(node) {
                        for (let i = node.childNodes.length - 1; i >= 0; i--) {
                            const child = node.childNodes[i];
                            if (child.nodeType === 8) {
                                child.remove();
                            } else if (child.nodeType === 1) {
                                removeComments(child);
                            }
                        }
                    }
                    removeComments(document.body);

                    // Run Readability
                    const documentClone = document.cloneNode(true);
                    const reader = new Readability(documentClone, {
                        maxElemsToParse: 0,
                        nbTopCandidates: 5,
                        charThreshold: 500
                    });
                    const article = reader.parse();

                    if (article && article.content) {
                        return JSON.stringify({
                            title: article.title || document.title || '',
                            content: article.content,
                            textContent: article.textContent || '',
                            excerpt: article.excerpt || ''
                        });
                    } else {
                        return JSON.stringify({
                            title: document.title || '',
                            content: '',
                            error: 'Readability returned null'
                        });
                    }
                } catch(e) {
                    return JSON.stringify({
                        title: document.title || '',
                        content: '',
                        error: e.toString()
                    });
                }
            })()
        "#;

        let result = tab.evaluate(article_js, false)
            .context("Failed to run Readability")?;

        let parsed: serde_json::Value = result.value
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or(serde_json::json!({"title": "", "content": ""}));

        let title = parsed["title"].as_str().unwrap_or("").to_string();
        let clean_html = parsed["content"].as_str().unwrap_or("").to_string();

        if let Some(error) = parsed.get("error") {
            println!("DEBUG: Readability error: {}", error);
        }
        println!("DEBUG: Title: {}", &title);
        println!("DEBUG: Content length: {} chars", clean_html.len());

        // Convert to Markdown
        let markdown = html2md::parse_html(&clean_html);

        Ok((markdown, title, clean_html))
    }
}
