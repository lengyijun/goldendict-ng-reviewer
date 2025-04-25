import mdict_utils
import mdict_utils.reader
from bs4 import BeautifulSoup
import sqlite3


# help(mdict_utils)
mdict_file_path = "/var/important_data/mdict-cli-rs/七大词典/韦氏/韦氏大学＋韦氏同义词二合一/Merriam-Webster's Collegiate Dictionary and Thesaurus, 2015.mdx"

conn = sqlite3.connect('merriam.db')
cursor = conn.cursor()
cursor.execute('''
CREATE TABLE IF NOT EXISTS merriam (
    word TEXT NOT NULL PRIMARY KEY,
    related_words TEXT NOT NULL
)
''')


# Iterate over the keys
for key in mdict_utils.reader.get_keys(mdict_file_path):
    html_content = mdict_utils.reader.query(mdict_file_path, key)
    # Step 4: Parse HTML and extract href attributes
    soup = BeautifulSoup(html_content, 'html.parser')
    all_elements = soup.find_all(class_='kud')
    s = set()
    for element in all_elements:
        links = element.find_all('a')
        hrefs = [link.get('href') for link in links if link.get('href')]

        # Print all href attributes
        for href in hrefs:
            href = href[8:]
            s.add(href)

    s.discard(key)

    if len(s) != 0:
        print(key)
        x = ",".join(s)
        try:
            cursor.execute('''
                INSERT INTO merriam (word, related_words) VALUES (?, ?)
            ''', (key, x))
        except sqlite3.Error as e:
            print(f"Error creating cursor: {e}")


conn.commit()
conn.close()
