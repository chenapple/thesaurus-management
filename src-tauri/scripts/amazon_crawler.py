#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Amazon 关键词排名爬虫
使用 cloudscraper 绕过反爬虫机制
"""

import sys
import json
import random
import time
import re
from urllib.parse import quote_plus
from datetime import datetime, timezone

try:
    import cloudscraper
    from bs4 import BeautifulSoup
except ImportError as e:
    print(json.dumps({
        "error": f"缺少依赖库: {e}. 请运行: pip install cloudscraper beautifulsoup4"
    }))
    sys.exit(1)


# 站点配置
COUNTRY_CONFIG = {
    "US": {
        "base_url": "https://www.amazon.com",
        "language": "en-US",
        "zipcode": "10001",
        "market_param": ""
    },
    "UK": {
        "base_url": "https://www.amazon.co.uk",
        "language": "en-GB",
        "zipcode": "SW1A 1AA",
        "market_param": ""
    },
    "DE": {
        "base_url": "https://www.amazon.de",
        "language": "de-DE",
        "zipcode": "10115",
        "market_param": "__mk_de_DE=%C3%85M%C3%85%C5%BD%C3%95%C3%91"
    },
    "FR": {
        "base_url": "https://www.amazon.fr",
        "language": "fr-FR",
        "zipcode": "75001",
        "market_param": "__mk_fr_FR=%C3%85M%C3%85%C5%BD%C3%95%C3%91"
    },
    "IT": {
        "base_url": "https://www.amazon.it",
        "language": "it-IT",
        "zipcode": "00100",
        "market_param": "__mk_it_IT=%C3%85M%C3%85%C5%BD%C3%95%C3%91"
    },
    "ES": {
        "base_url": "https://www.amazon.es",
        "language": "es-ES",
        "zipcode": "28001",
        "market_param": "__mk_es_ES=%C3%85M%C3%85%C5%BD%C3%95%C3%91"
    }
}

USER_AGENTS = [
    'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36',
    'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36',
    'Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:133.0) Gecko/20100101 Firefox/133.0',
    'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.1 Safari/605.1.15',
]


def get_random_user_agent():
    return random.choice(USER_AGENTS)


def search_keyword(keyword: str, target_asin: str, country: str, max_pages: int = 3) -> dict:
    """
    搜索关键词并返回目标ASIN的排名
    """
    config = COUNTRY_CONFIG.get(country, COUNTRY_CONFIG["US"])

    result = {
        "keyword": keyword,
        "target_asin": target_asin,
        "country": country,
        "organic_rank": None,
        "organic_page": None,
        "sponsored_rank": None,
        "sponsored_page": None,
        "product_info": None,
        "organic_top_50": [],
        "sponsored_top_20": [],
        "checked_at": datetime.now(timezone.utc).isoformat().replace("+00:00", "Z"),
        "error": None
    }

    # 构建搜索URL
    encoded_keyword = quote_plus(keyword)
    market_param = f"&{config['market_param']}" if config['market_param'] else ""
    search_url = f"{config['base_url']}/s?k={encoded_keyword}{market_param}"

    headers = {
        'User-Agent': get_random_user_agent(),
        'Accept': 'text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8',
        'Accept-Language': f"{config['language']},{config['language'].split('-')[0]};q=0.9",
        'Accept-Encoding': 'gzip, deflate, br',
        'Connection': 'keep-alive',
        'Upgrade-Insecure-Requests': '1',
        'Sec-Fetch-Dest': 'document',
        'Sec-Fetch-Mode': 'navigate',
        'Sec-Fetch-Site': 'same-origin',
        'Sec-Fetch-User': '?1',
        'Cache-Control': 'max-age=0',
        'Referer': config['base_url'] + '/',
    }

    organic_position = 0
    sponsored_position = 0
    found_organic = False
    found_sponsored = False

    try:
        # 创建 cloudscraper session
        session = cloudscraper.create_scraper(
            browser={
                'browser': 'chrome',
                'platform': 'windows',
                'desktop': True
            }
        )

        # 步骤1: 访问主页建立session
        try:
            home_response = session.get(
                config['base_url'],
                headers=headers,
                timeout=15
            )
        except Exception as e:
            result["error"] = f"访问主页失败: {e}"
            return result

        # 步骤2: 设置配送地址
        try:
            address_data = {
                'locationType': 'LOCATION_INPUT',
                'zipCode': config['zipcode'],
                'deviceType': 'web',
                'pageType': 's',
                'actionSource': 'glow',
            }

            address_headers = headers.copy()
            address_headers.update({
                'Content-Type': 'application/x-www-form-urlencoded',
                'X-Requested-With': 'XMLHttpRequest',
                'Referer': config['base_url'] + '/',
                'Origin': config['base_url'],
            })

            session.post(
                f"{config['base_url']}/gp/delivery/ajax/address-change.html",
                data=address_data,
                headers=address_headers,
                timeout=15
            )
        except Exception:
            pass  # 即使失败也继续

        # 等待一下
        time.sleep(random.uniform(1, 2))

        # 爬取搜索结果页面
        for page in range(1, max_pages + 1):
            if found_organic and found_sponsored:
                break

            page_url = f"{search_url}&page={page}" if page > 1 else search_url

            try:
                response = session.get(
                    page_url,
                    headers=headers,
                    timeout=30,
                    allow_redirects=True
                )

                if response.status_code == 503:
                    result["error"] = "Amazon 返回 503 错误，请稍后再试"
                    break

                if response.status_code != 200:
                    result["error"] = f"HTTP 错误: {response.status_code}"
                    break

                html = response.text

                # 检查机器人验证
                if 'Robot Check' in html or 'Type the characters' in html or 'captcha' in html.lower():
                    result["error"] = "检测到验证码页面，请稍后再试"
                    break

                soup = BeautifulSoup(html, 'html.parser')

                # 解析搜索结果
                search_results = soup.find_all('div', {'data-component-type': 's-search-result'})

                seen_asins = set()

                for item in search_results:
                    asin = item.get('data-asin', '')

                    if not asin or len(asin) != 10 or asin in seen_asins:
                        continue

                    seen_asins.add(asin)

                    # 检查是否是广告
                    is_sponsored = bool(item.find('span', string=re.compile(r'Sponsored|Sponsorisé|Gesponsert|Sponsorizzato|Patrocinado', re.IGNORECASE)))

                    if is_sponsored:
                        if not found_sponsored:
                            sponsored_position += 1

                            if len(result['sponsored_top_20']) < 20:
                                result['sponsored_top_20'].append(asin)

                            if asin.upper() == target_asin.upper():
                                result['sponsored_rank'] = sponsored_position
                                result['sponsored_page'] = page
                                found_sponsored = True
                    else:
                        if not found_organic:
                            organic_position += 1

                            if len(result['organic_top_50']) < 50:
                                result['organic_top_50'].append(asin)

                            if asin.upper() == target_asin.upper():
                                result['organic_rank'] = organic_position
                                result['organic_page'] = page
                                found_organic = True

                                # 提取产品信息
                                product_info = extract_product_info(item)
                                if product_info:
                                    result['product_info'] = product_info

                # 页面间延迟
                if page < max_pages and not (found_organic and found_sponsored):
                    time.sleep(random.uniform(2, 4))

            except Exception as e:
                result["error"] = f"第 {page} 页获取失败: {e}"
                break

        # 如果搜索结果中没有找到产品信息，从详情页获取
        if result['product_info'] is None:
            time.sleep(random.uniform(1, 2))
            product_info = fetch_product_detail(session, target_asin, config, headers)
            # 只有当成功获取到有效信息时才设置
            if product_info.get('title') or product_info.get('image_url') or product_info.get('price'):
                result['product_info'] = product_info

    except Exception as e:
        result["error"] = f"爬取失败: {e}"

    return result


def extract_product_info(item) -> dict:
    """从搜索结果项中提取产品信息"""
    info = {
        "asin": item.get('data-asin', ''),
        "title": None,
        "price": None,
        "rating": None,
        "reviews_count": None,
        "image_url": None
    }

    # 标题
    title_elem = item.find('h2')
    if title_elem:
        info['title'] = title_elem.get_text(strip=True)

    # 价格 - 使用更精确的选择器
    price_container = item.find('span', class_='a-price')
    if price_container:
        # 优先从 a-price 容器内的 a-offscreen 获取完整价格
        price_elem = price_container.find('span', class_='a-offscreen')
        if price_elem:
            info['price'] = price_elem.get_text(strip=True)
        else:
            # 备选：从 a-price-whole 和 a-price-fraction 组合
            whole = price_container.find('span', class_='a-price-whole')
            fraction = price_container.find('span', class_='a-price-fraction')
            symbol = price_container.find('span', class_='a-price-symbol')
            if whole:
                price_text = whole.get_text(strip=True).rstrip(',').rstrip('.')
                if fraction:
                    price_text += ',' + fraction.get_text(strip=True)
                if symbol:
                    info['price'] = symbol.get_text(strip=True) + price_text
                else:
                    info['price'] = price_text

    # 评分
    rating_elem = item.find('span', class_='a-icon-alt')
    if rating_elem:
        rating_text = rating_elem.get_text(strip=True)
        match = re.search(r'(\d+[.,]?\d*)', rating_text)
        if match:
            try:
                info['rating'] = float(match.group(1).replace(',', '.'))
            except:
                pass

    # 评论数 - 从 aria-label 属性获取（最可靠）
    reviews_count = None

    # 方法1: 从链接的 aria-label 属性获取 (如 "220 évaluations", "3 599 avis")
    reviews_link = item.find('a', href=re.compile(r'customerReviews|#customerReviews'))
    if reviews_link:
        aria_label = reviews_link.get('aria-label', '')
        if aria_label:
            # 移除千位分隔符并提取数字
            nums = re.sub(r'[\s.,\u00a0\u202f]', '', aria_label)
            nums = re.sub(r'[^\d]', '', nums)
            if nums:
                try:
                    reviews_count = int(nums)
                except:
                    pass

    # 方法2: 从链接文本获取 (如 "(220)", "(3 599)")
    if not reviews_count and reviews_link:
        link_text = reviews_link.get_text(strip=True)
        # 移除括号和千位分隔符
        nums = re.sub(r'[\s.,\u00a0\u202f\(\)]', '', link_text)
        nums = re.sub(r'[^\d]', '', nums)
        if nums:
            try:
                reviews_count = int(nums)
            except:
                pass

    info['reviews_count'] = reviews_count

    # 图片
    img_elem = item.find('img', class_='s-image')
    if img_elem:
        info['image_url'] = img_elem.get('src')

    return info


def fetch_product_detail(session, asin: str, config: dict, headers: dict) -> dict:
    """从产品详情页获取产品信息（当搜索结果中找不到时使用）"""
    info = {
        "asin": asin,
        "title": None,
        "price": None,
        "rating": None,
        "reviews_count": None,
        "image_url": None
    }

    try:
        detail_url = f"{config['base_url']}/dp/{asin}"
        response = session.get(
            detail_url,
            headers=headers,
            timeout=30,
            allow_redirects=True
        )

        if response.status_code != 200:
            return info

        html = response.text

        # 检查机器人验证
        if 'Robot Check' in html or 'Type the characters' in html or 'captcha' in html.lower():
            return info

        soup = BeautifulSoup(html, 'html.parser')

        # 标题
        title_elem = soup.find('span', id='productTitle')
        if title_elem:
            info['title'] = title_elem.get_text(strip=True)

        # 价格 - 从主价格区域获取，避免获取到运费等其他价格
        # 优先级：corePrice_feature_div > corePriceDisplay > apex_desktop > 全局搜索
        price_containers = [
            soup.find('div', id='corePrice_feature_div'),
            soup.find('div', id='corePriceDisplay_desktop_feature_div'),
            soup.find('div', id='apex_desktop'),
            soup.find('div', id='buybox'),
        ]

        for container in price_containers:
            if container and not info['price']:
                # 在容器内查找 a-price
                price_elem = container.find('span', class_='a-price')
                if price_elem:
                    price_offscreen = price_elem.find('span', class_='a-offscreen')
                    if price_offscreen:
                        info['price'] = price_offscreen.get_text(strip=True)
                        break
                    else:
                        whole = price_elem.find('span', class_='a-price-whole')
                        fraction = price_elem.find('span', class_='a-price-fraction')
                        symbol = price_elem.find('span', class_='a-price-symbol')
                        if whole:
                            price_text = whole.get_text(strip=True).rstrip(',').rstrip('.')
                            if fraction:
                                price_text += ',' + fraction.get_text(strip=True)
                            if symbol:
                                info['price'] = symbol.get_text(strip=True) + price_text
                            else:
                                info['price'] = price_text
                            break

        # 备选：旧版价格选择器
        if not info['price']:
            for selector_id in ['priceblock_ourprice', 'priceblock_dealprice', 'priceblock_saleprice']:
                price_elem = soup.find('span', id=selector_id)
                if price_elem:
                    price_text = price_elem.get_text(strip=True)
                    if price_text and any(c.isdigit() for c in price_text):
                        info['price'] = price_text
                        break

        # 最后备选：从页面全局找第一个看起来像主价格的 a-price
        # 但要排除明显是小价格的（如运费通常较小）
        if not info['price']:
            all_prices = soup.find_all('span', class_='a-price')
            for price_elem in all_prices:
                # 跳过被划掉的价格（通常有 a-text-price 类）
                if 'a-text-price' in price_elem.get('class', []):
                    continue
                price_offscreen = price_elem.find('span', class_='a-offscreen')
                if price_offscreen:
                    price_text = price_offscreen.get_text(strip=True)
                    # 提取数字部分判断是否是合理的主价格（通常 > 5）
                    nums = re.sub(r'[^\d,.]', '', price_text)
                    nums = nums.replace(',', '.')
                    try:
                        price_val = float(nums)
                        if price_val > 5:  # 主产品价格通常 > 5
                            info['price'] = price_text
                            break
                    except:
                        pass

        # 评分
        rating_elem = soup.find('span', class_='a-icon-alt')
        if rating_elem:
            rating_text = rating_elem.get_text(strip=True)
            match = re.search(r'(\d+[.,]?\d*)', rating_text)
            if match:
                try:
                    info['rating'] = float(match.group(1).replace(',', '.'))
                except:
                    pass

        # 评论数
        reviews_elem = soup.find('span', id='acrCustomerReviewText')
        if reviews_elem:
            reviews_text = reviews_elem.get_text(strip=True)
            # 移除千位分隔符并提取数字
            nums = re.sub(r'[\s.,\u00a0\u202f]', '', reviews_text)
            nums = re.sub(r'[^\d]', '', nums)
            if nums:
                try:
                    info['reviews_count'] = int(nums)
                except:
                    pass

        # 图片 - 主图
        img_elem = soup.find('img', id='landingImage')
        if not img_elem:
            img_elem = soup.find('img', id='imgBlkFront')
        if img_elem:
            # 优先使用 data-old-hires 或 data-a-dynamic-image
            img_url = img_elem.get('data-old-hires') or img_elem.get('src')
            if img_url:
                info['image_url'] = img_url

    except Exception as e:
        pass  # 静默失败，返回空信息

    return info


def main():
    if len(sys.argv) < 4:
        print(json.dumps({
            "error": "用法: python amazon_crawler.py <keyword> <asin> <country> [max_pages]"
        }))
        sys.exit(1)

    keyword = sys.argv[1]
    target_asin = sys.argv[2]
    country = sys.argv[3].upper()
    max_pages = int(sys.argv[4]) if len(sys.argv) > 4 else 3

    result = search_keyword(keyword, target_asin, country, max_pages)
    # Windows 默认使用 GBK 编码，强制使用 UTF-8
    output = json.dumps(result, ensure_ascii=False)
    sys.stdout.buffer.write(output.encode('utf-8'))
    sys.stdout.buffer.write(b'\n')


if __name__ == "__main__":
    main()
