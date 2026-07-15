# TockOS

[![tock-ci](https://github.com/tock/tock/actions/workflows/ci.yml/badge.svg)](https://github.com/tock/tock/actions/workflows/ci.yml)
[![slack](https://img.shields.io/badge/slack-tockos-informational)](https://join.slack.com/t/tockos/shared_invite/enQtNDE5ODQyNDU4NTE1LWVjNTgzMTMwYzA1NDI1MjExZjljMjFmOTMxMGIwOGJlMjk0ZTI4YzY0NTYzNWM0ZmJmZGFjYmY5MTJiMDBlOTk)
[![book](https://img.shields.io/badge/book-Tock_Book-green)](https://book.tockos.org)

Tock, Cortex-M və RISC-V əsaslı gömülü platformlarda bir-birinə etibar etməyən
bir neçə paralel tətbiqi işlətmək üçün nəzərdə tutulmuş gömülü əməliyyat
sistemidir. Tock-un dizaynı həm potensial zərərli tətbiqlərdən, həm də cihaz
sürücülərindən qorunmağı mərkəzə alır. Tock əməliyyat sisteminin müxtəlif
komponentlərini qorumaq üçün iki mexanizmdən istifadə edir. Birincisi, nüvə və
cihaz sürücüləri, kompilyasiya zamanı yaddaş təhlükəsizliyi və tip təhlükəsizliyi
təmin edən sistem proqramlaşdırma dili olan Rust ilə yazılmışdır. Tock, nüvəni
(məsələn, planlaşdırıcı və aparat abstraksiya qatı) platforma-spesifik cihaz
sürücülərindən qorumaq və cihaz sürücülərini bir-birindən təcrid etmək üçün
Rust-dan istifadə edir. İkincisi, Tock tətbiqləri bir-birindən və nüvədən
təcrid etmək üçün yaddaş qoruma vahidlərindən istifadə edir.

## Tock 2.x!

Tock artıq ikinci əsas buraxılışındadır! Ən son yeni xüsusiyyətlər və
təkmilləşdirmələr üçün [dəyişiklik jurnalına](CHANGELOG.md) baxın.

## Başlanğıc

Tock haqqında məlumat əldə etmək, layihəyə töhfə vermək və kömək almaq üçün
müxtəlif resurslar mövcuddur.

- Tock Haqqında
  * [Tock Kitabı](https://book.tockos.org): onlayn dərsliklər və sənədlər
  * [Təhlükəsiz Gömülü Sistemlərə Başlanğıc](https://link.springer.com/book/10.1007/978-1-4842-7789-8): Tock dərs kitabı
- Tock-u İnkişaf Etdirmək
  * [Tock API Sənədləri](https://docs.tockos.org)
  * [Töhfə Bələdçisi](.github/CONTRIBUTING.md)
  * [Kod İcmal Qaydaları](doc/CodeReview.md)
- Kömək Almaq
  * [Slack Kanalı](https://join.slack.com/t/tockos/shared_invite/enQtNDE5ODQyNDU4NTE1LWVjNTgzMTMwYzA1NDI1MjExZjljMjFmOTMxMGIwOGJlMjk0ZTI4YzY0NTYzNWM0ZmJmZGFjYmY5MTJiMDBlOTk)
  * [E-poçt Siyahısı](https://lists.tockos.org)
  * [Tock Bloqu](https://www.tockos.org/blog/)
  * [@talkingtock](https://twitter.com/talkingtock)

## Davranış Qaydaları

Tock layihəsi Rust [Davranış Qaydaları](https://www.rust-lang.org/conduct.html)'na riayət edir.

Bütün töhfəçilərin, icma üzvlərinin və ziyarətçilərin Davranış Qaydaları ilə
tanış olmaları və bu standartları depolar, söhbət kanalları və görüş tədbirləri
daxil olmaqla bütün Tock-a bağlı mühitlərdə izləmələri gözlənilir. Moderasiya
məsələləri üçün @tock/core-wg üzvləri ilə əlaqə saxlayın.

## Bu Layihəyə İstinad

#### Tock SOSP'17-də Təqdim Edilib

Amit Levy, Bradford Campbell, Branden Ghena, Daniel B. Giffin, Pat Pannuto,
Prabal Dutta və Philip Levis. 2017. Multiprogramming a 64kB Computer Safely
and Efficiently. 26-cı Əməliyyat Sistemləri Prinsipləri Simpoziumunun
Materiallarında (SOSP '17). Association for Computing Machinery, New York,
NY, USA, 234–251. DOI: https://doi.org/10.1145/3132747.3132786

**Bibtex**

```
@inproceedings{levy17multiprogramming,
      title = {Multiprogramming a 64kB Computer Safely and Efficiently},
      booktitle = {Proceedings of the 26th Symposium on Operating Systems Principles},
      series = {SOSP'17},
      year = {2017},
      month = {10},
      isbn = {978-1-4503-5085-3},
      location = {Shanghai, China},
      pages = {234--251},
      numpages = {18},
      url = {http://doi.acm.org/10.1145/3132747.3132786},
      doi = {10.1145/3132747.3132786},
      acmid = {3132786},
      publisher = {ACM},
      address = {New York, NY, USA},
      conference-url = {https://www.sigops.org/sosp/sosp17/},
      author = {Levy, Amit and Campbell, Bradford and Ghena, Branden and Giffin, Daniel B. and Pannuto, Pat and Dutta, Prabal and Levis, Philip},
}
```

## Lisenziya

Aşağıdakılardan biri əsasında lisenziyalaşdırılmışdır:

- Apache Lisenziyası, Versiya 2.0 ([LICENSE-APACHE](LICENSE-APACHE) və ya
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT lisenziyası ([LICENSE-MIT](LICENSE-MIT) və ya
  http://opensource.org/licenses/MIT)

Seçiminizə görə istifadə edə bilərsiniz.

Açıq şəkildə başqa cür bildirməsəniz, Apache-2.0 lisenziyasında müəyyən
edildiyi kimi işə daxil edilmək üçün qəsdən göndərilən hər hansı töhfə,
əlavə şərtlər olmadan yuxarıdakı kimi ikiqat lisenziyalaşdırılacaq.
