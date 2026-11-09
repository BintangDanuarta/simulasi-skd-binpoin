import re
from collections import Counter
from pathlib import Path

REPORT_DIR = Path(__file__).resolve().parent
SOURCE_PATH = REPORT_DIR / 'kandidat-stem-mungkin-terpotong.html'
OUTPUT_PATH = REPORT_DIR / 'soal_valid_terverifikasi.html'

with SOURCE_PATH.open('r', encoding='utf-8') as f:
    html = f.read()

# Daftar 375 ID Valid yang telah diverifikasi
valid_ids = {
    '1110','1685','2067','2069','2070','2071','2073','2075','2117','2118','2123','2124','2131','2135','2142','2145',
    '2172','2174','2176','2179','2180','2181','2184','2185','2186','2187','2188','2189','2190','2192','2193','2194',
    '2196','2199','2203','2207','2208','2209','2212','2214','2215','2224','2226','2227','2229','2233','2234','2235',
    '2236','2239','2241','2244','2249','2250','2252','2254','2258','2259','2260','2262','2267','2268','2270','2274',
    '2275','2276','2278','2279','2283','2284','2286','2290','4599','4600','4601','4606','4609','4623','4625','4626',
    '4639','4641','4642','4643','4648','4649','4657','5046','5049','5052','5053','5056','5060','5063','5064','5065',
    '5308','5311','5318','5323','5391','5394','5402','5404','5406','5417','6352','6358','6367','6368','6371','6374',
    '6376','6380','6387','6402','6403','6409','6410','6414','6416','6447','6448','6451','6452','6455','6458','6463',
    '6468','6470','6474','6484','6486','6489','6493','6496','6497','6500','6502','6504','6513','6527','6531','6540',
    '6541','6543','6547','7168','7172','7187','7188','7191','7193','7196','7197','7268','7269','7273','7280','7282',
    '7284','7285','7291','7296','7372','7373','7374','7376','7383','7384','7388','7389','7392','7395','7396','7397',
    '7467','7468','7471','7476','7477','7480','7482','7485','7488','7492','7493','7496','7566','7574','7577','7578',
    '7579','7580','7582','7586','7592','7594','7669','7678','7680','7687','7690','7765','7767','7769','7772','7775',
    '7776','7786','7788','7791','7867','7868','7871','7873','7875','7880','7881','7887',
    '1770','1774','1788','6576','6582','6583','6607','6609','6610','6623','6626','6633','6634','6637','6643','6651',
    '6665','6668','6670','6675','6683','6685','6687','6690','6694','6695','6697','6700','6701','6704','6709','6729',
    '6735','6736','6739','6743','6746','6747','6754','6787','6800','6801','6805','6806','6807','6808','7132','7133',
    '7147','7150','7160','7228','7237','7263','7266','7267','7328','7341','7344','7345','7347','7354','7364','7428',
    '7441','7452','7457','7460','7461','7522','7557','7561','7626','7632','7633','7634','7648','7656','7728','7729',
    '7737','7744','7752','7816','7822','7826','7830','7833','7838','7846','7853',
    '648','652','672','1719','1720','1721','1722','1724','1725','1726','1727','1729','1730','1731','1732','1733',
    '1734','1750','1753','1763','2095','2106','2107','6984','7028','7111','7204','7214','7305','7306','7307','7308',
    '7313','7409','7507','7510','7601','7604','7607','7612','7698','7704','7706',
    '736','744','751','756','828','842','846','852','2643','2648','2658','2664','2665','2669','2672','2675','2676',
    '2677','2820','2821','2822','2823','2827','2830','2832','3383','3384','3385','3387','3388','3389','3619','3697',
    '3707','3708','4367','4974','4977','4978','5969','5970','5971','5972','5973','5975','5977','6045','6058','6063',
    '6067','6068'
}

# Ekstrak tiap elemen kartu artikel soal
articles = re.findall(r'<article class="question".*?</article>', html, re.DOTALL)
filtered_articles = []
matched_ids = set()
category_counts = Counter()
for a in articles:
    match = re.search(r'<strong>#(\d+)</strong>', a)
    if not match:
        continue
    qid = match.group(1)
    if qid in valid_ids:
        filtered_articles.append(a)
        matched_ids.add(qid)
        category_match = re.search(r'data-category="([^"]+)"', a)
        if category_match:
            category_counts[category_match.group(1)] += 1

missing_ids = valid_ids - matched_ids
if missing_ids:
    raise ValueError(f'ID valid tidak ditemukan di HTML sumber: {sorted(missing_ids, key=int)}')

if len(matched_ids) != len(valid_ids):
    raise ValueError(f'Ada ID ganda: daftar={len(valid_ids)}, hasil unik={len(matched_ids)}')

# Ganti isi section kartu saja agar controls, script, dan struktur halaman tetap utuh.
list_pattern = re.compile(r'(<section id="list">).*?(</section>)', re.DOTALL)
output_html, replaced = list_pattern.subn(
    lambda match: match.group(1) + '\n' + '\n'.join(filtered_articles) + '\n' + match.group(2),
    html,
    count=1,
)
if replaced != 1:
    raise ValueError('Tidak menemukan tepat satu section id="list" di laporan sumber')

# Jumlah dihitung dari ID yang diberikan dan kartu yang cocok, bukan angka komentar lama.
category_order = ('TWK', 'TIU', 'TKP', 'CAMPUR')
category_summary = ' · '.join(
    f'{category}: {category_counts[category]}'
    for category in category_order
    if category_counts[category]
)
summary_pattern = re.compile(r'<p class="summary">.*?</p>', re.DOTALL)
summary = (
    f'<p class="summary">Hasil kurasi dari daftar ID yang diberikan: '
    f'{len(matched_ids)} soal cocok. Per kategori: {category_summary}. '
    'Daftar ini mengikuti ID pilihan pengguna; status valid belum merupakan verifikasi akademis independen.</p>'
)
output_html, replaced = summary_pattern.subn(summary, output_html, count=1)
if replaced != 1:
    raise ValueError('Tidak menemukan ringkasan laporan sumber')

with OUTPUT_PATH.open('w', encoding='utf-8') as f:
    f.write(output_html)

print(f'Selesai: {OUTPUT_PATH}')
print(f'Kartu terpilih: {len(filtered_articles)}; ID unik terpilih: {len(matched_ids)}')
print(f'Per kategori: {dict(category_counts)}')