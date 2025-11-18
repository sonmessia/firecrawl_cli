---
url: https://docs.firecrawl.dev/api-reference/endpoint/scrape
---

[Skip to main content](https://docs.firecrawl.dev/api-reference/endpoint/scrape#content-area)

[Firecrawl Docs home page![light logo](https://mintcdn.com/firecrawl/iilnMwCX-8eR1yOO/logo/logo.png?fit=max&auto=format&n=iilnMwCX-8eR1yOO&q=85&s=c45b3c967c19a39190e76fe8e9c2ed5a)![dark logo](https://mintcdn.com/firecrawl/iilnMwCX-8eR1yOO/logo/logo-dark.png?fit=max&auto=format&n=iilnMwCX-8eR1yOO&q=85&s=3fee4abe033bd3c26e8ad92043a91c17)](https://firecrawl.dev/)

v2

![US](https://d3gk2c5xim1je2.cloudfront.net/flags/US.svg)

English

Search...

⌘K

Search...

Navigation

Scrape Endpoints

Scrape

[Documentation](https://docs.firecrawl.dev/introduction)
[SDKs](https://docs.firecrawl.dev/sdks/overview)
[Integrations](https://www.firecrawl.dev/app)
[API Reference](https://docs.firecrawl.dev/api-reference/v2-introduction)

*   [Playground](https://firecrawl.dev/playground)
    
*   [Blog](https://firecrawl.dev/blog)
    
*   [Community](https://discord.gg/gSmWdAkdwd)
    
*   [Changelog](https://firecrawl.dev/changelog)
    

##### Using the API

*   [Introduction](https://docs.firecrawl.dev/api-reference/v2-introduction)
    

##### Scrape Endpoints

*   [POST\
    \
    Scrape](https://docs.firecrawl.dev/api-reference/endpoint/scrape)
    
*   [POST\
    \
    Batch Scrape](https://docs.firecrawl.dev/api-reference/endpoint/batch-scrape)
    
*   [GET\
    \
    Get Batch Scrape Status](https://docs.firecrawl.dev/api-reference/endpoint/batch-scrape-get)
    
*   [DEL\
    \
    Cancel Batch Scrape](https://docs.firecrawl.dev/api-reference/endpoint/batch-scrape-delete)
    
*   [GET\
    \
    Get Batch Scrape Errors](https://docs.firecrawl.dev/api-reference/endpoint/batch-scrape-get-errors)
    

##### Search Endpoints

*   [POST\
    \
    Search](https://docs.firecrawl.dev/api-reference/endpoint/search)
    

##### Map Endpoints

*   [POST\
    \
    Map](https://docs.firecrawl.dev/api-reference/endpoint/map)
    

##### Crawl Endpoints

*   [POST\
    \
    Crawl](https://docs.firecrawl.dev/api-reference/endpoint/crawl-post)
    
*   [GET\
    \
    Get Crawl Status](https://docs.firecrawl.dev/api-reference/endpoint/crawl-get)
    
*   [POST\
    \
    Crawl Params Preview](https://docs.firecrawl.dev/api-reference/endpoint/crawl-params-preview)
    
*   [DEL\
    \
    Cancel Crawl](https://docs.firecrawl.dev/api-reference/endpoint/crawl-delete)
    
*   [GET\
    \
    Get Crawl Errors](https://docs.firecrawl.dev/api-reference/endpoint/crawl-get-errors)
    
*   [GET\
    \
    Get Active Crawls](https://docs.firecrawl.dev/api-reference/endpoint/crawl-active)
    

##### Extract Endpoints

*   [POST\
    \
    Extract](https://docs.firecrawl.dev/api-reference/endpoint/extract)
    
*   [GET\
    \
    Get Extract Status](https://docs.firecrawl.dev/api-reference/endpoint/extract-get)
    

##### Account Endpoints

*   [GET\
    \
    Credit Usage](https://docs.firecrawl.dev/api-reference/endpoint/credit-usage)
    
*   [GET\
    \
    Historical Credit Usage](https://docs.firecrawl.dev/api-reference/endpoint/credit-usage-historical)
    
*   [GET\
    \
    Token Usage](https://docs.firecrawl.dev/api-reference/endpoint/token-usage)
    
*   [GET\
    \
    Historical Token Usage](https://docs.firecrawl.dev/api-reference/endpoint/token-usage-historical)
    
*   [GET\
    \
    Queue Status](https://docs.firecrawl.dev/api-reference/endpoint/queue-status)
    

Scrape a single URL and optionally extract information using an LLM

cURL

Copy

Ask AI

    curl --request POST \
      --url https://api.firecrawl.dev/v2/scrape \
      --header 'Authorization: Bearer <token>' \
      --header 'Content-Type: application/json' \
      --data '{
      "url": "<string>",
      "formats": [\
        "markdown"\
      ],
      "onlyMainContent": true,
      "includeTags": [\
        "<string>"\
      ],
      "excludeTags": [\
        "<string>"\
      ],
      "maxAge": 172800000,
      "headers": {},
      "waitFor": 0,
      "mobile": false,
      "skipTlsVerification": true,
      "timeout": 123,
      "parsers": [\
        "pdf"\
      ],
      "actions": [\
        {\
          "type": "wait",\
          "milliseconds": 2,\
          "selector": "#my-element"\
        }\
      ],
      "location": {
        "country": "US",
        "languages": [\
          "en-US"\
        ]
      },
      "removeBase64Images": true,
      "blockAds": true,
      "proxy": "auto",
      "storeInCache": true,
      "zeroDataRetention": false
    }'

200

402

429

500

Copy

Ask AI

    {
      "success": true,
      "data": {
        "markdown": "<string>",
        "summary": "<string>",
        "html": "<string>",
        "rawHtml": "<string>",
        "screenshot": "<string>",
        "links": [\
          "<string>"\
        ],
        "actions": {
          "screenshots": [\
            "<string>"\
          ],
          "scrapes": [\
            {\
              "url": "<string>",\
              "html": "<string>"\
            }\
          ],
          "javascriptReturns": [\
            {\
              "type": "<string>",\
              "value": "<any>"\
            }\
          ],
          "pdfs": [\
            "<string>"\
          ]
        },
        "metadata": {
          "title": "<string>",
          "description": "<string>",
          "language": "<string>",
          "sourceURL": "<string>",
          "keywords": "<string>",
          "ogLocaleAlternate": [\
            "<string>"\
          ],
          "<any other metadata> ": "<string>",
          "statusCode": 123,
          "error": "<string>"
        },
        "warning": "<string>",
        "changeTracking": {
          "previousScrapeAt": "2023-11-07T05:31:56Z",
          "changeStatus": "new",
          "visibility": "visible",
          "diff": "<string>",
          "json": {}
        },
        "branding": {}
      }
    }

POST

/

scrape

Try it

Scrape a single URL and optionally extract information using an LLM

cURL

Copy

Ask AI

    curl --request POST \
      --url https://api.firecrawl.dev/v2/scrape \
      --header 'Authorization: Bearer <token>' \
      --header 'Content-Type: application/json' \
      --data '{
      "url": "<string>",
      "formats": [\
        "markdown"\
      ],
      "onlyMainContent": true,
      "includeTags": [\
        "<string>"\
      ],
      "excludeTags": [\
        "<string>"\
      ],
      "maxAge": 172800000,
      "headers": {},
      "waitFor": 0,
      "mobile": false,
      "skipTlsVerification": true,
      "timeout": 123,
      "parsers": [\
        "pdf"\
      ],
      "actions": [\
        {\
          "type": "wait",\
          "milliseconds": 2,\
          "selector": "#my-element"\
        }\
      ],
      "location": {
        "country": "US",
        "languages": [\
          "en-US"\
        ]
      },
      "removeBase64Images": true,
      "blockAds": true,
      "proxy": "auto",
      "storeInCache": true,
      "zeroDataRetention": false
    }'

200

402

429

500

Copy

Ask AI

    {
      "success": true,
      "data": {
        "markdown": "<string>",
        "summary": "<string>",
        "html": "<string>",
        "rawHtml": "<string>",
        "screenshot": "<string>",
        "links": [\
          "<string>"\
        ],
        "actions": {
          "screenshots": [\
            "<string>"\
          ],
          "scrapes": [\
            {\
              "url": "<string>",\
              "html": "<string>"\
            }\
          ],
          "javascriptReturns": [\
            {\
              "type": "<string>",\
              "value": "<any>"\
            }\
          ],
          "pdfs": [\
            "<string>"\
          ]
        },
        "metadata": {
          "title": "<string>",
          "description": "<string>",
          "language": "<string>",
          "sourceURL": "<string>",
          "keywords": "<string>",
          "ogLocaleAlternate": [\
            "<string>"\
          ],
          "<any other metadata> ": "<string>",
          "statusCode": 123,
          "error": "<string>"
        },
        "warning": "<string>",
        "changeTracking": {
          "previousScrapeAt": "2023-11-07T05:31:56Z",
          "changeStatus": "new",
          "visibility": "visible",
          "diff": "<string>",
          "json": {}
        },
        "branding": {}
      }
    }

[​](https://docs.firecrawl.dev/api-reference/endpoint/scrape#what%E2%80%99s-new-in-v2)

What’s New in v2
----------------------------------------------------------------------------------------------------------

### 

[​](https://docs.firecrawl.dev/api-reference/endpoint/scrape#new-formats)

New Formats

*   `"summary"` - Get a concise summary of the page content
*   JSON extraction now uses object format: `{ type: "json", prompt, schema }`
*   Screenshot format now uses object format: `{ type: "screenshot", fullPage, quality, viewport }`
*   `"images"` - Extract all image URLs from the page
*   `"branding"` - Extract brand identity including colors, fonts, typography, spacing, and UI components

### 

[​](https://docs.firecrawl.dev/api-reference/endpoint/scrape#key-improvements)

Key Improvements

*   **Faster by default**: Requests are cached with `maxAge` defaulting to 2 days
*   **Sensible defaults**: `blockAds`, `skipTlsVerification`, and `removeBase64Images` are enabled by default
*   **Enhanced screenshot options**: Full control over screenshot parameters using object format

#### Authorizations

[​](https://docs.firecrawl.dev/api-reference/endpoint/scrape#authorization-authorization)

Authorization

string

header

required

Bearer authentication header of the form `Bearer <token>`, where `<token>` is your auth token.

#### Body

application/json

[​](https://docs.firecrawl.dev/api-reference/endpoint/scrape#body-url)

url

string<uri>

required

The URL to scrape

[​](https://docs.firecrawl.dev/api-reference/endpoint/scrape#body-formats)

formats

(Markdown · object | Summary · object | HTML · object | Raw HTML · object | Links · object | Images · object | Screenshot · object | JSON · object | Change Tracking · object | Branding · object)\[\]

Output formats to include in the response. You can specify one or more formats, either as strings (e.g., `'markdown'`) or as objects with additional options (e.g., `{ type: 'json', schema: {...} }`). Some formats require specific options to be set. Example: `['markdown', { type: 'json', schema: {...} }]`.

*   Markdown
    
*   Summary
    
*   HTML
    
*   Raw HTML
    
*   Links
    
*   Images
    
*   Screenshot
    
*   JSON
    
*   Change Tracking
    
*   Branding
    

Show child attributes

[​](https://docs.firecrawl.dev/api-reference/endpoint/scrape#body-only-main-content)

onlyMainContent

boolean

default:true

Only return the main content of the page excluding headers, navs, footers, etc.

[​](https://docs.firecrawl.dev/api-reference/endpoint/scrape#body-include-tags)

includeTags

string\[\]

Tags to include in the output.

[​](https://docs.firecrawl.dev/api-reference/endpoint/scrape#body-exclude-tags)

excludeTags

string\[\]

Tags to exclude from the output.

[​](https://docs.firecrawl.dev/api-reference/endpoint/scrape#body-max-age)

maxAge

integer

default:172800000

Returns a cached version of the page if it is younger than this age in milliseconds. If a cached version of the page is older than this value, the page will be scraped. If you do not need extremely fresh data, enabling this can speed up your scrapes by 500%. Defaults to 2 days.

[​](https://docs.firecrawl.dev/api-reference/endpoint/scrape#body-headers)

headers

object

Headers to send with the request. Can be used to send cookies, user-agent, etc.

[​](https://docs.firecrawl.dev/api-reference/endpoint/scrape#body-wait-for)

waitFor

integer

default:0

Specify a delay in milliseconds before fetching the content, allowing the page sufficient time to load.

[​](https://docs.firecrawl.dev/api-reference/endpoint/scrape#body-mobile)

mobile

boolean

default:false

Set to true if you want to emulate scraping from a mobile device. Useful for testing responsive pages and taking mobile screenshots.

[​](https://docs.firecrawl.dev/api-reference/endpoint/scrape#body-skip-tls-verification)

skipTlsVerification

boolean

default:true

Skip TLS certificate verification when making requests

[​](https://docs.firecrawl.dev/api-reference/endpoint/scrape#body-timeout)

timeout

integer

Timeout in milliseconds for the request.

[​](https://docs.firecrawl.dev/api-reference/endpoint/scrape#body-parsers)

parsers

array

Controls how files are processed during scraping. When "pdf" is included (default), the PDF content is extracted and converted to markdown format, with billing based on the number of pages (1 credit per page). When an empty array is passed, the PDF file is returned in base64 encoding with a flat rate of 1 credit total.

Show child attributes

[​](https://docs.firecrawl.dev/api-reference/endpoint/scrape#body-actions)

actions

(Wait · object | Screenshot · object | Click · object | Write text · object | Press a key · object | Scroll · object | Scrape · object | Execute JavaScript · object | Generate PDF · object)\[\]

Actions to perform on the page before grabbing the content

*   Wait
    
*   Screenshot
    
*   Click
    
*   Write text
    
*   Press a key
    
*   Scroll
    
*   Scrape
    
*   Execute JavaScript
    
*   Generate PDF
    

Show child attributes

[​](https://docs.firecrawl.dev/api-reference/endpoint/scrape#body-location)

location

object

Location settings for the request. When specified, this will use an appropriate proxy if available and emulate the corresponding language and timezone settings. Defaults to 'US' if not specified.

Show child attributes

[​](https://docs.firecrawl.dev/api-reference/endpoint/scrape#body-remove-base64-images)

removeBase64Images

boolean

default:true

Removes all base 64 images from the output, which may be overwhelmingly long. The image's alt text remains in the output, but the URL is replaced with a placeholder.

[​](https://docs.firecrawl.dev/api-reference/endpoint/scrape#body-block-ads)

blockAds

boolean

default:true

Enables ad-blocking and cookie popup blocking.

[​](https://docs.firecrawl.dev/api-reference/endpoint/scrape#body-proxy)

proxy

enum<string>

default:auto

Specifies the type of proxy to use.

*   **basic**: Proxies for scraping sites with none to basic anti-bot solutions. Fast and usually works.
*   **stealth**: Stealth proxies for scraping sites with advanced anti-bot solutions. Slower, but more reliable on certain sites. Costs up to 5 credits per request.
*   **auto**: Firecrawl will automatically retry scraping with stealth proxies if the basic proxy fails. If the retry with stealth is successful, 5 credits will be billed for the scrape. If the first attempt with basic is successful, only the regular cost will be billed.

If you do not specify a proxy, Firecrawl will default to auto.

Available options:

`basic`,

`stealth`,

`auto`

[​](https://docs.firecrawl.dev/api-reference/endpoint/scrape#body-store-in-cache)

storeInCache

boolean

default:true

If true, the page will be stored in the Firecrawl index and cache. Setting this to false is useful if your scraping activity may have data protection concerns. Using some parameters associated with sensitive scraping (actions, headers) will force this parameter to be false.

[​](https://docs.firecrawl.dev/api-reference/endpoint/scrape#body-zero-data-retention)

zeroDataRetention

boolean

default:false

If true, this will enable zero data retention for this scrape. To enable this feature, please contact [help@firecrawl.dev](mailto:help@firecrawl.dev)

#### Response

200

application/json

Successful response

[​](https://docs.firecrawl.dev/api-reference/endpoint/scrape#response-success)

success

boolean

[​](https://docs.firecrawl.dev/api-reference/endpoint/scrape#response-data)

data

object

Show child attributes

[Suggest edits](https://github.com/firecrawl/firecrawl-docs/edit/main/api-reference/endpoint/scrape.mdx)
[Raise issue](https://github.com/firecrawl/firecrawl-docs/issues/new?title=Issue%20on%20docs&body=Path:%20/api-reference/endpoint/scrape)

[Introduction\
\
Previous](https://docs.firecrawl.dev/api-reference/v2-introduction)
[Batch Scrape\
\
Next](https://docs.firecrawl.dev/api-reference/endpoint/batch-scrape)

⌘I