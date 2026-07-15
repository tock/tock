# TockOS

[![tock-ci](https://github.com/tock/tock/actions/workflows/ci.yml/badge.svg)](https://github.com/tock/tock/actions/workflows/ci.yml)
[![slack](https://img.shields.io/badge/slack-tockos-informational)](https://join.slack.com/t/tockos/shared_invite/enQtNDE5ODQyNDU4NTE1LWVjNTgzMTMwYzA1NDI1MjExZjljMjFmOTMxMGIwOGJlMjk0ZTI4YzY0NTYzNWM0ZmJmZGFjYmY5MTJiMDBlOTk)
[![book](https://img.shields.io/badge/book-Tock_Book-green)](https://book.tockos.org)

Tock, Cortex-M ve RISC-V tabanlı gömülü platformlarda birden fazla eşzamanlı,
birbirine güvenmeyen uygulamayı çalıştırmak için tasarlanmış gömülü bir işletim
sistemidir. Tock'un tasarımı, hem kötü amaçlı olabilecek uygulamalardan hem de
aygıt sürücülerinden korunmayı merkeze alır. Tock, işletim sisteminin farklı
bileşenlerini korumak için iki mekanizma kullanır. İlk olarak, çekirdek ve
aygıt sürücüleri, derleme zamanında bellek güvenliği ve tür güvenliği sağlayan
bir sistem programlama dili olan Rust ile yazılmıştır. Tock, çekirdeği
(örneğin zamanlayıcı ve donanım soyutlama katmanı) platforma özgü aygıt
sürücülerinden korumak ve aygıt sürücülerini birbirinden izole etmek için
Rust'ı kullanır. İkinci olarak Tock, uygulamaları birbirinden ve çekirdekten
izole etmek için bellek koruma birimleri kullanır.

## Tock 2.x!

Tock artık ikinci ana sürümünde! En son yeni özellikler ve iyileştirmeler için
[değişiklik günlüğüne](CHANGELOG.md) göz atın.

## Başlarken

Tock hakkında bilgi edinmek, projeye katkıda bulunmak ve yardım almak için
çeşitli kaynaklar mevcuttur.

- Tock Hakkında
  * [Tock Kitabı](https://book.tockos.org): çevrimiçi öğreticiler ve belgeler
  * [Güvenli Gömülü Sistemlerle Başlarken](https://link.springer.com/book/10.1007/978-1-4842-7789-8): Tock ders kitabı
- Tock Geliştirme
  * [Tock API Belgeleri](https://docs.tockos.org)
  * [Katkı Rehberi](.github/CONTRIBUTING.md)
  * [Kod İnceleme Yönergeleri](doc/CodeReview.md)
- Yardım Alma
  * [Slack Kanalı](https://join.slack.com/t/tockos/shared_invite/enQtNDE5ODQyNDU4NTE1LWVjNTgzMTMwYzA1NDI1MjExZjljMjFmOTMxMGIwOGJlMjk0ZTI4YzY0NTYzNWM0ZmJmZGFjYmY5MTJiMDBlOTk)
  * [E-posta Listesi](https://lists.tockos.org)
  * [Tock Blog](https://www.tockos.org/blog/)
  * [@talkingtock](https://twitter.com/talkingtock)

## Davranış Kuralları

Tock projesi, Rust [Davranış Kuralları](https://www.rust-lang.org/conduct.html)'na uymaktadır.

Tüm katkıda bulunanların, topluluk üyelerinin ve ziyaretçilerin Davranış
Kuralları ile tanışmaları ve bu standartları depolar, sohbet kanalları ve
buluşma etkinlikleri dahil olmak üzere tüm Tock bağlı ortamlarda takip etmeleri
beklenmektedir. Moderasyon sorunları için lütfen @tock/core-wg üyeleriyle
iletişime geçin.

## Bu Projeyi Atıfla

#### Tock, SOSP'17'de Sunulmuştur

Amit Levy, Bradford Campbell, Branden Ghena, Daniel B. Giffin, Pat Pannuto,
Prabal Dutta ve Philip Levis. 2017. Multiprogramming a 64kB Computer Safely
and Efficiently. 26. İşletim Sistemleri İlkeleri Sempozyumu Bildirilerinde
(SOSP '17). Association for Computing Machinery, New York, NY, USA, 234–251.
DOI: https://doi.org/10.1145/3132747.3132786

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

## Lisans

Aşağıdakilerden biri kapsamında lisanslanmıştır:

- Apache Lisansı, Sürüm 2.0 ([LICENSE-APACHE](LICENSE-APACHE) veya
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT lisansı ([LICENSE-MIT](LICENSE-MIT) veya
  http://opensource.org/licenses/MIT)

Tercihinize göre seçebilirsiniz.

Aksi açıkça belirtilmedikçe, Apache-2.0 lisansında tanımlandığı şekilde
çalışmaya dahil edilmek üzere kasıtlı olarak gönderilen her türlü katkı,
ek koşullar veya şartlar olmaksızın yukarıdaki gibi çift lisanslı olacaktır.
