[English](../../README.md) | [简体中文](../zh-cn/README_zh-CN.md) | **Español** | [Português (BR)](../pt-BR/README_pt-BR.md) | [日本語](../ja/README_ja.md) | [Русский](../ru/README_ru.md) | [Français](../fr/README_fr.md) | [Deutsch](../de/README_de.md) | [한국어](../ko/README_ko.md) | [Türkçe](../tr/README.tr.md) | [Bahasa Indonesia](../id/README.id.md)

# FamTree

Creado y mantenido por **[john3sobieski](https://github.com/john3sobieski)**. Repositorio: <https://github.com/john3sobieski/family-tree>

Una aplicación de escritorio para Windows para crear, editar y explorar tu árbol genealógico, y una aplicación para Android para verlo en el móvil. Todo vive en un único archivo `.famtree`: personas, relaciones y todas las fotos.

> **Estado:** lista para usar. El proyecto se considera terminado: a partir de ahora solo se esperan correcciones menores. Descarga el instalador de Windows o el APK de Android en [Releases](https://github.com/john3sobieski/family-tree/releases) (consulta [Instalación](#instalación)).

## Características

- **Tres modos**
  - **Vista** – mapa mental de solo lectura con zoom y desplazamiento fluidos.
  - **Edición** – añade y modifica personas, fotos y relaciones.
  - **Lista** – una tabla ordenable y filtrable con todas las personas del árbol.
- **Mapa mental construido a tu alrededor.** Marca a una persona como «yo» y el árbol se dibuja desde tus antepasados más lejanos hacia abajo, de modo que padres, hermanos, tíos, primos y parientes más lejanos aparecen automáticamente. Ambas ramas de la familia se colocan una junto a la otra, y también se incluyen las familias políticas.
- **Tarjetas con foto.** Cada tarjeta muestra una foto tipo carnet, el nombre, el apellido (con el apellido de soltera entre paréntesis en el caso de las mujeres), el patronímico, una franja de color para el género y hasta dos banderas de nacionalidad.
- **Ventana de persona.** Haz clic en una tarjeta para abrir una ventana más grande con la foto completa, una galería (más de 10 fotos por persona no es problema), fechas, edad, lugares de nacimiento y fallecimiento, profesión, notas y enlaces clicables a padres, parejas e hijos.
- **Búsqueda.** Escribe un nombre y la cámara vuela hasta esa persona en el mapa y la resalta. En el modo Lista funciona como filtro. `Ctrl+F` enfoca el campo de búsqueda.
- **Relaciones.** Padres, parejas, hijos y hermanos se pueden añadir con un clic o vincular a personas que ya existen. Los ciclos y los vínculos duplicados se rechazan.
- **Fechas incompletas.** ¿Solo conoces el año? Sin problema: cualquier parte de una fecha se puede dejar vacía.
- **Nacionalidades.** Elige hasta dos por persona de una lista integrada de 197 países (todos los miembros de la ONU más el Vaticano, Palestina, Taiwán y Kosovo). Las banderas las dibuja la propia aplicación.
- **Pensado para árboles grandes.** Solo se dibujan las tarjetas visibles, las miniaturas se cargan de forma diferida y el diseño solo se recalcula cuando cambia la estructura. Se ha probado con árboles de miles de personas.
- **Guardado automático.** Cuando el árbol ya tiene un archivo, cada cambio se guarda automáticamente un instante después de que dejes de escribir. Para un árbol nuevo, usa primero **Guardar como…**. Además, se te avisa antes de cerrar si algo no se pudo guardar.
- **Idiomas de la interfaz.** Русский, English, 中文, Español, Français, Deutsch y Polski. Cambia el idioma en el menú **Idioma**; la elección se recuerda. En el primer inicio, la aplicación sigue el idioma de visualización de Windows y, si no es uno de estos, usa inglés. Los nombres de los países también se traducen.
- **Exportación.** **Archivo → Exportar…** guarda todo el árbol como PNG o como PDF vectorial, con márgenes en los bordes. En PNG se elige el fondo (transparente, blanco o negro) y colores oscuros o claros; el tamaño se calcula solo para que todo se vea nítido (hasta ×2). El PDF se puede ampliar sin perder calidad. Los árboles muy grandes es mejor exportarlos a PDF.
- **Doble clic para abrir.** En el menú **Archivo** puedes asociar los archivos `.famtree` con la aplicación.
- **Tema oscuro y claro.** Se cambia con el botón de sol/luna a la derecha de la barra de estado; la elección se recuerda.
- **Aplicación para Android (solo visualización).** Abre el mismo archivo `.famtree` en tu móvil o tablet. El mapa se ve exactamente igual que en el PC (misma disposición, tarjetas, banderas y líneas) y admite arrastrar y hacer zoom con los dedos, búsqueda con vuelo de la cámara hasta la persona, una página de persona a pantalla completa con fotos y enlaces a familiares, **Ajustar** y tema claro/oscuro. El archivo nunca se modifica: la edición, el modo «Lista» y la exportación se quedan en el PC. La interfaz sigue el idioma del teléfono (los mismos siete idiomas; si no, inglés). Consulta [Instalación → Android](#android).

## Instalación

### Windows

La aplicación funciona en Windows 10 y Windows 11. Es el mismo instalador y el mismo exe para ambos.

1. Abre la página de [Releases](https://github.com/john3sobieski/family-tree/releases) y descarga **`FamTree-Setup-<versión>.exe`**.
2. Ejecútalo. Se instala solo para tu usuario, así que no hacen falta permisos de administrador. El asistente ofrece abrir los archivos `.famtree` con FamTree y crear un acceso directo en el escritorio.
3. Inicia **FamTree** desde el menú Inicio.

Notas:

- Windows puede mostrar una advertencia de SmartScreen porque el instalador y el `.exe` no están firmados. Elige *Más información → Ejecutar de todas formas*.
- Para actualizar, ejecuta un Setup más reciente sobre la versión anterior. Tus árboles y ajustes se conservan.
- Para desinstalar, usa *Configuración → Aplicaciones*. Tus archivos `.famtree` nunca se borran.

### Android

La aplicación para Android solo muestra árboles y nunca modifica tu archivo. Necesita Android 10 o superior en un dispositivo ARM de 64 bits (prácticamente todos los móviles y tablets actuales).

1. Descarga **`FamTree-<versión>.apk`** desde la página [Releases](https://github.com/john3sobieski/family-tree/releases) directamente en tu dispositivo.
2. Abre el archivo. Android te pedirá una sola vez que permitas instalar aplicaciones desde ese origen (tu navegador o gestor de archivos); confírmalo.
3. Pasa tu archivo `.famtree` al dispositivo (almacenamiento en la nube, mensajería, cable USB). Después ábrelo desde un gestor de archivos con **Abrir con → FamTree**, o inicia FamTree y toca **Abrir** para elegir el archivo. Se recuerda el último archivo abierto. Si abres otro `.famtree` mientras la aplicación está en marcha, se carga sin reiniciar.

Notas:

- La aplicación solo se distribuye como APK, no a través de Google Play. Play Protect puede avisar de una aplicación de un desarrollador desconocido; elige instalar de todos modos.
- Para poder reabrirlo, la aplicación guarda una copia privada del último archivo abierto; el original nunca se toca.

### Compilarlo tú mismo

También puedes obtener el instalador de GitHub Actions sin instalar ninguna herramienta (hace falta una cuenta de GitHub): haz un fork del [repositorio](https://github.com/john3sobieski/family-tree), abre **Actions → Build → Run workflow** y descarga el artefacto **`famtree-setup`** de la ejecución terminada (se conserva 90 días por defecto).

El APK de Android se genera de la misma forma: **Actions → Android → Run workflow** y descarga el artefacto **`famtree-android-<versión>`**.

## Inicio rápido

1. Abre `famtree.exe`. Usa **Archivo → Nuevo** para empezar tu árbol, o **Archivo → Abrir** para abrir uno existente.
2. Cambia al modo **Edición** y añade a la primera persona (**＋ Nueva persona**). En un árbol vacío, esa persona pasa a ser automáticamente «yo».
3. Selecciona a una persona y usa **＋ Padre/madre**, **＋ Pareja**, **＋ Hijo/a** o **＋ Hermano/a** para hacer crecer el árbol. Rellena nombres, fechas, fotos y notas a la derecha.
4. Elige **Guardar como…** y ponle un nombre. A partir de ahí, los cambios se guardan automáticamente.
5. Cambia a **Vista** para explorar el mapa: arrastra para desplazarte, usa la rueda para hacer zoom, **Ajustar** para verlo todo y haz clic en una tarjeta para ver los detalles.

**En Android:** abre un archivo `.famtree` (consulta [Instalación → Android](#android)), arrastra para desplazarte y junta o separa dos dedos para hacer zoom. **Ajustar** muestra todo de una vez, la lupa abre la búsqueda de personas y al tocar una tarjeta se abre la página de la persona (toca un familiar para saltar a él). El botón o gesto Atrás del sistema cierra la página o la búsqueda paso a paso; en el mapa envía la aplicación a segundo plano. El botón con sol/luna cambia el tema (por defecto la aplicación sigue el tema del sistema).

## El archivo `.famtree`

Un archivo `.famtree` es un archivo ZIP normal:

```
tree.json            people and relationships (plain, human-readable text)
media/<hash>.jpg     photos, resized to at most 1600 px
thumb/<hash>.jpg     small thumbnails for cards
```

- Las fotos se guardan dentro del archivo, así que puedes enviar un único `.famtree` a un familiar sin perder nada. Las fotos idénticas se almacenan una sola vez.
- `tree.json` se puede leer en cualquier editor de texto y contiene comentarios que describen cada relación. Si quieres editarlo a mano, descomprime el archivo, edita el fichero y vuelve a empaquetarlo. Editarlo directamente dentro del `.famtree` corromperá el archivo.
- Los archivos antiguos se siguen abriendo en las versiones nuevas de la aplicación.

## Limitaciones conocidas

- Las traducciones de la interfaz (salvo el ruso) aún no las han revisado hablantes nativos. Las correcciones son bienvenidas a través de los issues.
- Las fechas siempre se muestran como `DD.MM.AAAA`, sea cual sea el idioma.
- Todavía no hay deshacer.
- Importar muchas fotos de golpe puede congelar la ventana un momento.
- Las fotos HEIC no son compatibles (JPEG, PNG, WebP y BMP sí lo son).
- La miniatura de una tarjeta es un recorte centrado, y todavía no se puede elegir el área de recorte.
- En familias muy grandes, el diseño aún puede colocar a las personas en sitios extraños o cruzar líneas entre ramas (consulta la hoja de ruta).
- Las nacionalidades no se muestran en el modo Lista.
- Android: solo visualización (sin edición, modo «Lista» ni exportación), dispositivos ARM de 64 bits con Android 10 o superior, solo APK (no está en Google Play).
- Android: de momento las fotos no se pueden ampliar con el gesto de pellizco, y no hay menú de idioma: la aplicación sigue el idioma del sistema.

## Hoja de ruta

El proyecto está terminado: la aplicación de Windows y el visor de Android están completos. A partir de ahora solo se planean correcciones menores, sobre todo para fallos poco frecuentes de la disposición (personas en lugares extraños en familias muy grandes).

## Compilar desde el código fuente

Si prefieres compilar en local, necesitas la toolchain de Rust en Windows:

```
cargo test --release
cargo build --release
```

El resultado es `target/release/famtree.exe`. El proyecto usa [egui/eframe](https://github.com/emilk/egui) para la interfaz.

La aplicación de Android (`android/`) se genera únicamente con el workflow de GitHub Actions `.github/workflows/android.yml` (cargo-apk, Android SDK/NDK, una pequeña capa de Java y la firma). Compila el código de escritorio de `src/` (`model`, `container`, `layout`, `view`, `flags` y otros), por eso el mapa es idéntico en ambas plataformas. Para firmar el APK con tu propia clave, añade los secretos del repositorio `ANDROID_KEYSTORE_BASE64`, `ANDROID_KEYSTORE_PASSWORD`, `ANDROID_KEY_ALIAS` y, si es distinta, `ANDROID_KEY_PASSWORD`; sin ellos el workflow usa una clave temporal.

## Informes de errores

¿Has encontrado un problema, sobre todo con cómo se colocan las personas en el mapa? Por favor, [abre un issue](https://github.com/john3sobieski/family-tree/issues) y, si puedes, adjunta un pequeño archivo `.famtree` que lo reproduzca (sin datos personales reales, si el árbol es privado). Para la aplicación de Android, indica también tu versión de Android y el modelo del dispositivo.

## Licencia

FamTree se distribuye bajo la [PolyForm Noncommercial License 1.0.0](https://polyformproject.org/licenses/noncommercial/1.0.0/). Consulta [`LICENSE.md`](LICENSE.md).

En resumen:

- Puedes usar, copiar, modificar y compartir la aplicación y su código fuente con fines **no comerciales** (uso personal, historia familiar, afición, educación, investigación y similares).
- **No está permitido venderla ni usarla comercialmente.**
- Siempre que compartas la aplicación o una versión modificada (un fork, una recompilación, una redistribución), debes conservar los términos de la licencia y la línea `Required Notice` de `LICENSE.md`, que acredita a john3sobieski como autor.

Este resumen no constituye asesoramiento legal; lo que cuenta es el texto de la licencia.
