[English](README.md) | [简体中文](README.zh-CN.md) | **Español** | [Português (BR)](README.pt-BR.md) | [日本語](README.ja.md) | [Русский](README.ru.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [한국어](README.ko.md) | [Türkçe](README.tr.md) | [Bahasa Indonesia](README.id.md)

# FamTree

Creado y mantenido por **[john3sobieski](https://github.com/john3sobieski)**. Repositorio: <https://github.com/john3sobieski/family-tree>

Una aplicación de escritorio para Windows para crear, editar y explorar tu árbol genealógico. Todo vive en un único archivo `.famtree`: personas, relaciones y todas las fotos.

> **Estado:** desarrollo temprano. Todavía no hay versiones listas para descargar, y la aplicación solo está disponible compilándola con GitHub Actions (consulta [Instalación](#instalación)).

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
- **Doble clic para abrir.** En el menú **Archivo** puedes asociar los archivos `.famtree` con la aplicación.

## Instalación

Por ahora la aplicación solo funciona en Windows 10 y se compila desde el código fuente con GitHub Actions, así que no necesitas Rust ni ninguna otra herramienta en tu equipo. Eso sí, necesitas una cuenta de GitHub.

1. Haz un **fork** del [repositorio](https://github.com/john3sobieski/family-tree) (o usa tu propia copia).
2. En tu fork, abre la pestaña **Actions**. Si GitHub te lo pide, habilita los workflows del fork.
3. Selecciona el workflow **Build**, haz clic en **Run workflow** y espera a que termine (unos 2–3 minutos). También se ejecuta en cada push.
4. Abre la ejecución terminada y descarga el artefacto **`famtree-windows`** desde la sección *Artifacts* al final de la página.
5. Descomprímelo. Dentro hay un `famtree.exe`. No hay nada que instalar, simplemente ejecútalo.

Notas:

- Windows puede mostrar una advertencia de SmartScreen porque el `.exe` no está firmado. Elige *Más información → Ejecutar de todas formas* si confías en tu propia compilación.
- GitHub conserva los artefactos durante un tiempo limitado (90 días por defecto), así que guarda el `.exe` en un lugar permanente.
- Si mueves `famtree.exe`, vuelve a pulsar el botón de asociación de `.famtree` en el menú **Archivo**, porque la asociación apunta a la ubicación del exe.

Cuando exista un instalador de verdad, se publicará en **Releases** y esta sección se reemplazará.

## Inicio rápido

1. Abre `famtree.exe`. Usa **Archivo → Demo** para ver un árbol de ejemplo, o **Archivo → Nuevo** para empezar el tuyo.
2. Cambia al modo **Edición** y añade a la primera persona (**＋ Nueva persona**). En un árbol vacío, esa persona pasa a ser automáticamente «yo».
3. Selecciona a una persona y usa **＋ Padre/madre**, **＋ Pareja**, **＋ Hijo/a** o **＋ Hermano/a** para hacer crecer el árbol. Rellena nombres, fechas, fotos y notas a la derecha.
4. Elige **Guardar como…** y ponle un nombre. A partir de ahí, los cambios se guardan automáticamente.
5. Cambia a **Vista** para explorar el mapa: arrastra para desplazarte, usa la rueda para hacer zoom, **Ajustar** para verlo todo y haz clic en una tarjeta para ver los detalles.

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
- Solo Windows 10.
- Todavía no hay deshacer ni exportación a PNG/PDF.
- Importar muchas fotos de golpe puede congelar la ventana un momento.
- Las fotos HEIC no son compatibles (JPEG, PNG, WebP y BMP sí lo son).
- La miniatura de una tarjeta es un recorte centrado, y todavía no se puede elegir el área de recorte.
- En familias muy grandes, el diseño aún puede colocar a las personas en sitios extraños o cruzar líneas entre ramas (consulta la hoja de ruta).
- Las nacionalidades no se muestran en el modo Lista.

## Hoja de ruta

Mejoras previstas, sin ningún orden concreto:

- [ ] **Un instalador de verdad**, en lugar de un `.exe` suelto de Actions, publicado en Releases.
- [ ] **Corrección de errores de diseño**: de vez en cuando hay personas que acaban en un lugar equivocado del mapa.

## Compilar desde el código fuente

Si prefieres compilar en local, necesitas la toolchain de Rust en Windows:

```
cargo test --release
cargo build --release
```

El resultado es `target/release/famtree.exe`. El proyecto usa [egui/eframe](https://github.com/emilk/egui) para la interfaz.

## Informes de errores

¿Has encontrado un problema, sobre todo con cómo se colocan las personas en el mapa? Por favor, [abre un issue](https://github.com/john3sobieski/family-tree/issues) y, si puedes, adjunta un pequeño archivo `.famtree` que lo reproduzca (sin datos personales reales, si el árbol es privado).

## Licencia

FamTree se distribuye bajo la [PolyForm Noncommercial License 1.0.0](https://polyformproject.org/licenses/noncommercial/1.0.0/). Consulta [`LICENSE.md`](LICENSE.md).

En resumen:

- Puedes usar, copiar, modificar y compartir la aplicación y su código fuente con fines **no comerciales** (uso personal, historia familiar, afición, educación, investigación y similares).
- **No está permitido venderla ni usarla comercialmente.**
- Siempre que compartas la aplicación o una versión modificada (un fork, una recompilación, una redistribución), debes conservar los términos de la licencia y la línea `Required Notice` de `LICENSE.md`, que acredita a john3sobieski como autor.

Este resumen no constituye asesoramiento legal; lo que cuenta es el texto de la licencia.
