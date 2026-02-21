import ipaddress
from urllib.parse import urlparse

import feedparser
import requests
from bs4 import BeautifulSoup
from duckduckgo_search import DDGS
from langchain.tools import tool


BLOCKED_HOSTS = {"localhost", "127.0.0.1", "::1", "0.0.0.0", "host.docker.internal"}


def _is_private_or_loopback(hostname: str) -> bool:
    if not hostname:
        return True
    host = hostname.strip().lower()
    if host in BLOCKED_HOSTS:
        return True
    try:
        ip = ipaddress.ip_address(host)
        return ip.is_private or ip.is_loopback or ip.is_link_local or ip.is_reserved
    except ValueError:
        return False


def _validate_external_url(url: str) -> str:
    parsed = urlparse(url)
    if parsed.scheme not in {"http", "https"}:
        raise ValueError("Only http/https URLs are allowed.")
    if _is_private_or_loopback(parsed.hostname or ""):
        raise ValueError("Local/private network URLs are blocked for security.")
    return url


@tool
def search_web(query: str) -> str:
    """Searches the web using DuckDuckGo and returns the top 5 results."""
    try:
        results = []
        with DDGS() as ddgs:
            for record in ddgs.text(query, max_results=5):
                results.append(
                    f"Title: {record.get('title', '')}\n"
                    f"Link: {record.get('href', '')}\n"
                    f"Snippet: {record.get('body', '')}"
                )
        return "\n\n".join(results)
    except Exception as exc:
        return f"Error searching web: {exc}"


@tool
def read_rss_feed(feed_url: str) -> str:
    """Reads the latest headlines from an RSS feed. Returns titles and links."""
    try:
        safe_url = _validate_external_url(feed_url)
        feed = feedparser.parse(safe_url)
        if feed.bozo:
            return "Error: Could not parse this feed. It might be invalid or down."

        results = []
        for entry in feed.entries[:5]:
            title = entry.get("title", "No Title")
            link = entry.get("link", "#")
            results.append(f"- Title: {title}\n  Link: {link}")

        return "\n".join(results)
    except Exception as exc:
        return f"Error reading feed: {exc}"


@tool
def visit_webpage(url: str) -> str:
    """Visits a webpage and returns plain text content."""
    try:
        safe_url = _validate_external_url(url)
        headers = {
            "User-Agent": (
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) "
                "AppleWebKit/537.36 (KHTML, like Gecko) "
                "Chrome/122.0.0.0 Safari/537.36"
            )
        }
        response = requests.get(safe_url, headers=headers, timeout=(5, 10), allow_redirects=True)
        response.raise_for_status()

        soup = BeautifulSoup(response.text, "html.parser")
        for element in soup(["script", "style", "nav", "footer", "aside", "header", "form"]):
            element.decompose()

        text = soup.get_text(separator=" ", strip=True)
        return text[:8000]
    except Exception as exc:
        return f"Error visiting page: {exc}"
