from pathlib import Path
import urllib.parse
import re
import aiohttp

from bs4 import BeautifulSoup

CACHE_DIR = Path("cache")

GUROBI_REF_MAN_URL = urllib.parse.urlparse('https://www.gurobi.com/documentation/9.5/refman/')
DOC_REMOVE = [
    r"For examples of how to query or modify parameter values from our different APIs, refer to our Parameter Examples.",
    r"For examples of how to query or modify attributes, refer to our Attribute Examples.",
    r"Next:.+Up:.+Previous.+",
    r"One important note about integer-valued parameters: while the maximum value that can be stored in a signed integer is.+",
    r"Please refer to this section for more information on SOS constraints.",
    "next",
    "up",
    "previous",
]
DOC_REMOVE = [re.compile(r) for r in DOC_REMOVE]

def remove_newlines(s: str) -> str:
    return ' '.join(filter(bool, s.splitlines()))

def get_url(path: str):
    url = GUROBI_REF_MAN_URL._replace(path=GUROBI_REF_MAN_URL.path + path).geturl()
    return url

async def fetch_html(session: aiohttp.ClientSession, path: str):
    cache_path = CACHE_DIR / path
    if cache_path.exists():
        with open(cache_path, 'r') as fp:
            return fp.read()

    url = get_url(path)

    async with session.get(url) as res:
        print("GET", url, res.status)
        if res.status != 200:
            raise Exception("bad request")
        text = await res.text()

    cache_path.parent.mkdir(parents=True, exist_ok=True)
    with open(cache_path, 'w') as fp:
        fp.write(text)

    return text

def _clean_documentation(s: str) -> list:
    paragraphs = map(lambda x: remove_newlines(x.strip()), filter(bool, s.split('\n\n')))
    paragraphs = [
        p for p in paragraphs if not any(r.fullmatch(p) for r in DOC_REMOVE)
    ]

    return paragraphs

_DTYPES = {
    "double": "dbl",
    "string": "str",
    "int": "int",
    "char": "chr"
}

async def fetch_parameter_data(session: aiohttp.ClientSession, name: str, path: str) -> dict:
    doc = await fetch_html(session, path)
    soup = BeautifulSoup(doc, 'html.parser')
    replace_images_with_alt(soup)

    table = soup.find("table")
    data = {"name": name, "url": get_url(path) }

    ty = table.find(string="Type:", recursive=True).parent.parent.find_next_sibling('td').text
    data['ty'] = ty
    data['dtype'] = _DTYPES[ty]

    data['default'] = table.find(string="Default value:", recursive=True).parent.parent.find_next_sibling('td').text
    if ty == 'int' or ty == "double":
        data['min'] = table.find(string="Minimum value:", recursive=True).parent.parent.find_next_sibling('td').text
        data['max'] = table.find(string="Maximum value:", recursive=True).parent.parent.find_next_sibling('td').text


    documentation = _clean_documentation(table.find_next_sibling('p').text)
    data['cl_only'] = "Note: Command-line only" in documentation
    data['doc'] = documentation
    return data

_OTYPES = {
    "Model": "model",
    "Multi-objective": "model",
    "Multi-Scenario": "model",
    "Quality": "model",
    "Linear Constraint": "constr",
    "Quadratic Constraint": "qconstr",
    "SOS": "sos",
    "Variable": "var",
    "General Constraint": "gconstr",
    "Batch": "batch",
}

async def fetch_attribute_data(session: aiohttp.ClientSession, name: str, path: str) -> dict:
    doc = await fetch_html(session, path)
    soup = BeautifulSoup(doc, 'html.parser')
    replace_images_with_alt(soup)

    table = soup.find("table")
    data = {"name": name, "url": get_url(path) }

    ty = table.find(string="Type:", recursive=True).parent.parent.find_next_sibling('td').text
    mod = table.find(string="Modifiable:", recursive=True).parent.parent.find_next_sibling('td').text
    object_ty = table.parent\
        .find(class_='navigation')\
        .find('b', text=lambda x : "Up:" in x)\
        .find_next_sibling("a")\
        .text.replace(" Attributes", "")

    data['ty'] = ty
    data['dtype'] = _DTYPES[ty]
    data['otype'] = _OTYPES[object_ty]
    data['modifiable'] = mod.lower() == "yes"
    documentation = _clean_documentation(table.find_next_sibling('p').text)
    data['cl_only'] = "Note: Command-line only" in documentation
    data['doc'] = documentation
    return data

async def fetch_parameter_list(session: aiohttp.ClientSession):
    html_doc = await fetch_html(session, "parameters.html")
    soup = BeautifulSoup(html_doc, 'html.parser')
    desc = soup.find("a", string="Parameter Descriptions")
    items = desc.find_next_sibling('ul').find_all('a')
    return {i.text: i.attrs['href'] for i in items}

async def fetch_attribute_list(session: aiohttp.ClientSession):
    html_doc = await fetch_html(session, "attributes.html")
    soup = BeautifulSoup(html_doc, 'html.parser')
    attrlist = {}
    for i in soup.find('ul', class_="ChildLinks").find_all('a'):
        if 'Attributes' in i.text or 'Examples' in i.text:
            continue
        attrlist[i.text] = i.attrs['href']

    return attrlist

def http_session():
    http_headers = {'user-agent': "Mozilla/5.0 (X11; Linux x86_64; rv:78.0) Gecko/20100101 Firefox/78.0"}
    return aiohttp.ClientSession(connector=aiohttp.TCPConnector(limit=20), headers=http_headers)

def replace_images_with_alt(soup):
    for img in soup.find_all("img"):
        try:
            alt = img.attrs["alt"]
        except KeyError:
            continue

        img.replace_with(BeautifulSoup(alt, "html.parser"))

