"""
Tools available to the research agent.

Changes from original:
- Removed dead `search_web` (DuckDuckGo) tool — searching is handled in agent.py
  via `perform_dynamic_search` so it respects the user's chosen search engine.
- Added `extract_links` tool so the executor can pull URLs from a page for follow-up VISIT calls.
- Tightened timeouts and added a max content length guard on visit_webpage.
- SSRF protection retained and slightly improved (rejects empty hostnames explicitly).
"""

import ipaddress
from urllib.parse import urlparse

import feedparser
import requests
from bs4 import BeautifulSoup
from langchain.tools import tool

# ---------------------------------------------------------------------------
# Security helpers
# ---------------------------------------------------------------------------

BLOCKED_HOSTS = {
    "localhost",
    "127.0.0.1",
    "::1",
    "0.0.0.0",
    "host.docker.internal",
    "metadata.google.internal",  # GCP metadata endpoint
}

MAX_CONTENT_CHARS = 8_000


def _is_private_or_loopback(hostname: str) -> bool:
    """Return True if the hostname resolves to a private / loopback address."""
    if not hostname:
        return True
    host = hostname.strip().lower()
    if host in BLOCKED_HOSTS:
        return True
    # Strip brackets from IPv6 literals like [::1]
    host = host.strip("[]")
    try:
        ip = ipaddress.ip_address(host)
        return (
            ip.is_private
            or ip.is_loopback
            or ip.is_link_local
            or ip.is_reserved
            or ip.is_multicast
        )
    except ValueError:
        # Not an IP literal — hostname-based DNS rebinding is a known risk but
        # acceptable for a local desktop tool.
        return False


def _validate_external_url(url: str) -> str:
    """Raise ValueError if the URL is not a safe, external http/https URL."""
    url = url.strip()
    parsed = urlparse(url)
    if parsed.scheme not in {"http", "https"}:
        raise ValueError(f"Only http/https URLs are allowed (got '{parsed.scheme}').")
    if _is_private_or_loopback(parsed.hostname or ""):
        raise ValueError("Local/private network URLs are blocked for security.")
    return url


# ---------------------------------------------------------------------------
# Shared request helper
# ---------------------------------------------------------------------------

_HEADERS = {
    "User-Agent": (
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) "
        "AppleWebKit/537.36 (KHTML, like Gecko) "
        "Chrome/122.0.0.0 Safari/537.36"
    )
}


# ---------------------------------------------------------------------------
# Tools
# ---------------------------------------------------------------------------


@tool
def read_rss_feed(feed_url: str) -> str:
    """Read the latest headlines and links from an RSS/Atom feed URL.

    Returns up to 7 entries as 'Title — Link' pairs.
    """
    try:
        safe_url = _validate_external_url(feed_url)
        feed = feedparser.parse(safe_url)
        if feed.bozo and not feed.entries:
            return "Error: Could not parse this feed — it may be invalid or unavailable."

        lines: list[str] = []
        for entry in feed.entries[:7]:
            title = entry.get("title", "No Title").strip()
            link = entry.get("link", "")
            summary = entry.get("summary", "")[:120].strip()
            lines.append(f"- **{title}**\n  {link}\n  {summary}")

        return "\n\n".join(lines) if lines else "Feed is empty."
    except ValueError as exc:
        return f"URL validation error: {exc}"
    except Exception as exc:
        return f"Error reading RSS feed: {exc}"


@tool
def visit_webpage(url: str) -> str:
    """Fetch a webpage and return its cleaned plain-text content (up to 8 000 chars)."""
    try:
        safe_url = _validate_external_url(url)
        response = requests.get(
            safe_url,
            headers=_HEADERS,
            timeout=(5, 15),
            allow_redirects=True,
        )
        response.raise_for_status()

        soup = BeautifulSoup(response.text, "html.parser")
        # Remove boilerplate elements
        for tag in soup(["script", "style", "nav", "footer", "aside", "header", "form", "noscript"]):
            tag.decompose()

        text = soup.get_text(separator=" ", strip=True)

        # Collapse excessive whitespace
        import re
        text = re.sub(r"\s{3,}", "  ", text)

        return text[:MAX_CONTENT_CHARS]
    except ValueError as exc:
        return f"URL validation error: {exc}"
    except requests.HTTPError as exc:
        return f"HTTP error visiting page: {exc}"
    except Exception as exc:
        return f"Error visiting page: {exc}"


@tool
def extract_links(url: str) -> str:
    """Fetch a webpage and return up to 10 relevant hyperlinks found on the page.

    Useful when you need to discover URLs to VISIT next.
    """
    try:
        safe_url = _validate_external_url(url)
        response = requests.get(safe_url, headers=_HEADERS, timeout=(5, 10), allow_redirects=True)
        response.raise_for_status()

        soup = BeautifulSoup(response.text, "html.parser")
        base = f"{urlparse(safe_url).scheme}://{urlparse(safe_url).netloc}"

        links: list[str] = []
        seen: set[str] = set()
        for a in soup.find_all("a", href=True):
            href: str = a["href"].strip()
            if href.startswith("//"):
                href = "https:" + href
            elif href.startswith("/"):
                href = base + href
            if href.startswith("http") and href not in seen:
                seen.add(href)
                text = a.get_text(strip=True)[:60]
                links.append(f"- {text or '(no text)'}: {href}")
            if len(links) >= 10:
                break

        return "\n".join(links) if links else "No links found."
    except ValueError as exc:
        return f"URL validation error: {exc}"
    except Exception as exc:
        return f"Error extracting links: {exc}"
