[English](README.md) | [简体中文](README.zh-CN.md) | [Español](README.es.md) | [Português (BR)](README.pt-BR.md) | [日本語](README.ja.md) | [Русский](README.ru.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [한국어](README.ko.md) | [Türkçe](README.tr.md) | **Bahasa Indonesia**

# FamTree

Dibuat dan dikelola oleh **[john3sobieski](https://github.com/john3sobieski)**. Repositori: <https://github.com/john3sobieski/family-tree>

Aplikasi desktop untuk Windows guna membuat, mengedit, dan menjelajahi pohon keluarga Anda. Semuanya tersimpan dalam satu berkas `.famtree`: orang, hubungan, dan semua foto.

> **Status:** tahap pengembangan awal. Unduh installer di [Releases](https://github.com/john3sobieski/family-tree/releases) (lihat [Instalasi](#instalasi)).

## Fitur

- **Tiga mode**
  - **Lihat** (*View*) – peta pikiran (mind map) hanya-baca dengan zoom dan geser yang mulus.
  - **Edit** (*Edit*) – tambah dan ubah orang, foto, dan hubungan.
  - **Daftar** (*List*) – tabel yang bisa diurutkan dan difilter berisi semua orang di dalam pohon.
- **Peta pikiran yang dibangun di sekitar Anda.** Tandai satu orang sebagai "saya", dan pohon digambar dari leluhur teratas Anda ke bawah, sehingga orang tua, saudara kandung, paman, bibi, sepupu, dan kerabat yang lebih jauh muncul secara otomatis. Kedua sisi keluarga ditata berdampingan, dan keluarga dari pihak ipar/besan juga disertakan.
- **Kartu foto.** Setiap kartu menampilkan foto ala pas foto, nama, nama keluarga (nama gadis dalam tanda kurung untuk perempuan), patronimik, strip warna untuk jenis kelamin, dan hingga dua bendera kewarganegaraan.
- **Jendela orang.** Klik sebuah kartu untuk membuka jendela yang lebih besar dengan foto penuh, galeri (10+ foto per orang tidak masalah), tanggal, usia, tempat lahir dan wafat, profesi, catatan, serta tautan yang bisa diklik ke orang tua, pasangan, dan anak.
- **Pencarian.** Ketik sebuah nama, lalu kamera terbang ke orang tersebut di peta dan menyorotinya. Di mode Daftar, pencarian berfungsi sebagai filter. `Ctrl+F` memfokuskan kolom pencarian.
- **Hubungan.** Orang tua, pasangan, anak, dan saudara kandung bisa ditambahkan dengan satu klik atau ditautkan ke orang yang sudah ada. Siklus dan tautan ganda ditolak.
- **Tanggal tidak lengkap.** Hanya tahu tahunnya? Tidak masalah, bagian mana pun dari tanggal boleh dikosongkan.
- **Kewarganegaraan.** Pilih hingga dua per orang dari daftar bawaan berisi 197 negara (semua anggota PBB ditambah Vatikan, Palestina, Taiwan, dan Kosovo). Bendera digambar oleh aplikasi itu sendiri.
- **Dibuat untuk pohon besar.** Hanya kartu yang terlihat yang digambar, thumbnail dimuat secara lazy, dan tata letak hanya dihitung ulang saat struktur berubah. Sudah diuji dengan pohon berisi ribuan orang.
- **Simpan otomatis.** Setelah pohon memiliki berkas, setiap perubahan disimpan otomatis sesaat setelah Anda berhenti mengetik. Untuk pohon baru, gunakan **Simpan sebagai…** (*Save as…*) terlebih dahulu. Anda juga akan ditanya sebelum menutup jika ada yang tidak berhasil disimpan.
- **Bahasa antarmuka.** Русский, English, 中文, Español, Français, Deutsch, dan Polski. Ganti bahasa lewat menu **Language**; pilihan Anda diingat. Saat pertama kali dijalankan, aplikasi mengikuti bahasa tampilan Windows dan memakai bahasa Inggris jika bahasa tersebut tidak tersedia. Nama negara juga diterjemahkan.
- **Klik ganda untuk membuka.** Di menu **Berkas** (*File*), Anda bisa mengaitkan berkas `.famtree` dengan aplikasi.
- **Tema gelap dan terang.** Ganti dengan tombol matahari/bulan di sisi kanan bilah status; pilihan Anda diingat.

## Instalasi

Aplikasi berjalan di Windows 10 dan Windows 11. Installer dan exe-nya sama untuk keduanya.

1. Buka halaman [Releases](https://github.com/john3sobieski/family-tree/releases) dan unduh **`FamTree-Setup-<versi>.exe`**.
2. Jalankan. Instalasi hanya untuk pengguna Anda, jadi hak administrator tidak diperlukan. Wizard menawarkan untuk membuka berkas `.famtree` dengan FamTree dan menambahkan pintasan desktop.
3. Jalankan **FamTree** dari menu Start.

Catatan:

- Windows mungkin menampilkan peringatan SmartScreen karena installer dan `.exe` tidak ditandatangani. Pilih *Info lebih lanjut → Tetap jalankan*.
- Untuk memperbarui, jalankan Setup yang lebih baru di atas versi lama. Pohon dan pengaturan Anda tetap tersimpan.
- Untuk menghapus, gunakan *Pengaturan → Aplikasi*. Berkas `.famtree` Anda tidak pernah dihapus.
- Jika Anda memakai exe portabel dan memindahkannya, tekan lagi tombol asosiasi `.famtree` di menu **Berkas**.

### Membangun sendiri

Anda juga bisa mendapatkan installer dari GitHub Actions tanpa memasang alat apa pun (perlu akun GitHub): fork [repositori](https://github.com/john3sobieski/family-tree), buka **Actions → Build → Run workflow**, lalu unduh artefak **`famtree-setup`** dari run yang selesai (disimpan 90 hari secara bawaan).

## Mulai cepat

1. Jalankan `famtree.exe`. Gunakan **Berkas → Demo** (*File → Demo*) untuk melihat contoh pohon, atau **Berkas → Baru** (*File → New*) untuk memulai pohon Anda sendiri.
2. Beralih ke mode **Edit** dan tambahkan orang pertama (*＋ New person*). Di pohon kosong, orang tersebut otomatis menjadi "saya".
3. Pilih seseorang dan gunakan **＋ Orang tua**, **＋ Pasangan**, **＋ Anak**, atau **＋ Saudara** (*＋ Parent*, *＋ Partner*, *＋ Child*, *＋ Sibling*) untuk mengembangkan pohon. Isi nama, tanggal, foto, dan catatan di sebelah kanan.
4. Pilih **Simpan sebagai…** dan beri nama. Sejak saat itu, perubahan disimpan otomatis.
5. Beralih ke **Lihat** untuk menjelajahi peta: seret untuk menggeser, gulir untuk zoom, **Sesuaikan** (*Fit to view*) untuk melihat semuanya, dan klik kartu untuk melihat detail.

## Berkas `.famtree`

Berkas `.famtree` adalah arsip ZIP biasa:

```
tree.json            people and relationships (plain, human-readable text)
media/<hash>.jpg     photos, resized to at most 1600 px
thumb/<hash>.jpg     small thumbnails for cards
```

- Foto disimpan di dalam berkas, jadi Anda bisa mengirim satu berkas `.famtree` ke kerabat tanpa ada yang hilang. Foto yang identik hanya disimpan satu kali.
- `tree.json` dapat dibaca di editor teks apa pun dan memiliki komentar yang menjelaskan setiap hubungan. Jika ingin mengeditnya secara manual, ekstrak arsipnya, edit berkasnya, lalu kemas kembali. Mengeditnya langsung di dalam `.famtree` akan merusak arsip.
- Berkas lama tetap bisa dibuka di versi aplikasi yang lebih baru.

## Keterbatasan yang diketahui

- Terjemahan antarmuka (selain bahasa Rusia) belum diperiksa oleh penutur asli. Koreksi sangat disambut lewat issue.
- Tanggal selalu ditampilkan dalam format `DD.MM.YYYY`, apa pun bahasanya.
- Belum ada undo dan ekspor ke PNG/PDF.
- Mengimpor banyak foto sekaligus bisa membuat jendela membeku sesaat.
- Foto HEIC tidak didukung (JPEG, PNG, WebP, dan BMP didukung).
- Thumbnail pada kartu adalah potongan di bagian tengah, dan area potongan belum bisa dipilih.
- Pada keluarga yang sangat besar, tata letak masih bisa menempatkan orang di posisi yang aneh atau membuat garis antarcabang saling bersilangan (lihat peta jalan).
- Kewarganegaraan tidak ditampilkan di mode Daftar.

## Peta jalan

Perbaikan yang direncanakan, tanpa urutan tertentu:

- [ ] **Perbaikan bug tata letak**: sesekali orang berakhir di posisi yang salah pada peta.

## Membangun dari kode sumber

Jika Anda lebih suka membangun secara lokal, Anda memerlukan toolchain Rust di Windows:

```
cargo test --release
cargo build --release
```

Hasilnya adalah `target/release/famtree.exe`. Proyek ini menggunakan [egui/eframe](https://github.com/emilk/egui) untuk antarmukanya.

## Laporan bug

Menemukan masalah, terutama soal cara orang ditempatkan di peta? Silakan [buka issue](https://github.com/john3sobieski/family-tree/issues) dan, jika bisa, lampirkan berkas `.famtree` kecil yang mereproduksi masalahnya (tanpa data pribadi asli jika pohonnya bersifat privat).

## Lisensi

FamTree dilisensikan di bawah [PolyForm Noncommercial License 1.0.0](https://polyformproject.org/licenses/noncommercial/1.0.0/). Lihat [`LICENSE.md`](LICENSE.md).

Singkatnya:

- Anda boleh menggunakan, menyalin, memodifikasi, dan membagikan aplikasi serta kode sumbernya untuk tujuan **nonkomersial** (penggunaan pribadi, sejarah keluarga, hobi, pendidikan, penelitian, dan sejenisnya).
- **Menjual atau menggunakannya secara komersial tidak diizinkan.**
- Setiap kali Anda membagikan aplikasi atau versi yang dimodifikasi (fork, build ulang, distribusi ulang), Anda harus mempertahankan ketentuan lisensi dan baris `Required Notice` dari `LICENSE.md`, yang mencantumkan john3sobieski sebagai pembuat.

Ringkasan ini bukan nasihat hukum; yang berlaku adalah teks lisensinya.
