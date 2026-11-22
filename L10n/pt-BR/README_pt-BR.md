[English](README.md) | [简体中文](README.zh-CN.md) | [Español](README.es.md) | **Português (BR)** | [日本語](README.ja.md) | [Русский](README.ru.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [한국어](README.ko.md) | [Türkçe](README.tr.md) | [Bahasa Indonesia](README.id.md)

# FamTree

Criado e mantido por **[john3sobieski](https://github.com/john3sobieski)**. Repositório: <https://github.com/john3sobieski/family-tree>

Um aplicativo de desktop para Windows para montar, editar e navegar pela sua árvore genealógica. Tudo fica em um único arquivo `.famtree`: pessoas, relacionamentos e todas as fotos.

> **Status:** em desenvolvimento inicial. Ainda não há versões prontas para download, e o aplicativo só está disponível compilando-o pelo GitHub Actions (veja [Instalação](#instalação)).

## Recursos

- **Três modos**
  - **Visualização** (*View*) – mapa mental somente leitura, com zoom e arrastar suaves.
  - **Edição** (*Edit*) – adicione e altere pessoas, fotos e relacionamentos.
  - **Lista** (*List*) – uma tabela ordenável e filtrável com todas as pessoas da árvore.
- **Mapa mental construído em torno de você.** Marque uma pessoa como "eu" e a árvore é desenhada dos seus ancestrais mais antigos para baixo, então pais, irmãos, tios, primos e parentes mais distantes aparecem automaticamente. Os dois lados da família ficam lado a lado, e as famílias dos parentes por afinidade também entram.
- **Cartões com foto.** Cada cartão mostra uma foto estilo 3x4, o nome, o sobrenome (com o nome de solteira entre parênteses no caso das mulheres), o patronímico, uma faixa colorida indicando o gênero e até duas bandeiras de nacionalidade.
- **Janela da pessoa.** Clique em um cartão para abrir uma janela maior com a foto completa, uma galeria (mais de 10 fotos por pessoa não é problema), datas, idade, locais de nascimento e falecimento, profissão, anotações e links clicáveis para pais, parceiros e filhos.
- **Busca.** Digite um nome e a câmera voa até a pessoa no mapa e a destaca. No modo Lista, funciona como filtro. `Ctrl+F` coloca o foco no campo de busca.
- **Relacionamentos.** Pais, parceiros, filhos e irmãos podem ser adicionados com um clique ou vinculados a pessoas que já existem. Ciclos e vínculos duplicados são rejeitados.
- **Datas incompletas.** Só sabe o ano? Sem problema, qualquer parte de uma data pode ficar vazia.
- **Nacionalidades.** Escolha até duas por pessoa em uma lista integrada de 197 países (todos os membros da ONU, mais Vaticano, Palestina, Taiwan e Kosovo). As bandeiras são desenhadas pelo próprio aplicativo.
- **Feito para árvores grandes.** Só os cartões visíveis são desenhados, as miniaturas são carregadas sob demanda e o layout só é recalculado quando a estrutura muda. Já foi testado com árvores de milhares de pessoas.
- **Salvamento automático.** Quando a árvore já tem um arquivo, cada alteração é salva automaticamente um instante depois que você para de digitar. Para uma árvore nova, use **Salvar como…** (*Save as…*) primeiro. Você também é avisado antes de fechar se algo não pôde ser salvo.
- **Idiomas da interface.** Русский, English, 中文, Español, Français, Deutsch e Polski. Troque o idioma no menu **Language**; a escolha é lembrada. Na primeira execução, o aplicativo segue o idioma de exibição do Windows e, se ele não estiver entre esses, usa inglês. Os nomes dos países também são traduzidos.
- **Duplo clique para abrir.** No menu **Arquivo** (*File*) você pode associar os arquivos `.famtree` ao aplicativo.

## Instalação

Por enquanto o aplicativo funciona apenas no Windows 10 e é compilado a partir do código-fonte pelo GitHub Actions, então você não precisa de Rust nem de nenhuma outra ferramenta na sua máquina. Mas precisa de uma conta no GitHub.

1. Faça um **fork** do [repositório](https://github.com/john3sobieski/family-tree) (ou use a sua própria cópia).
2. No seu fork, abra a aba **Actions**. Se o GitHub pedir, habilite os workflows do fork.
3. Selecione o workflow **Build**, clique em **Run workflow** e espere terminar (cerca de 2–3 minutos). Ele também roda a cada push.
4. Abra a execução concluída e baixe o artefato **`famtree-windows`** na seção *Artifacts*, no final da página.
5. Descompacte. Dentro está o `famtree.exe`. Não há nada para instalar, é só executar.

Observações:

- O Windows pode exibir um aviso do SmartScreen porque o `.exe` não é assinado. Escolha *Mais informações → Executar assim mesmo* se você confia na sua própria compilação.
- O GitHub guarda os artefatos por tempo limitado (90 dias por padrão), então salve o `.exe` em um lugar permanente.
- Se você mover o `famtree.exe`, clique de novo no botão de associação do `.famtree` no menu **Arquivo**, porque a associação aponta para o local do exe.

Quando houver um instalador de verdade, ele será publicado em **Releases** e esta seção será substituída.

## Primeiros passos

1. Abra o `famtree.exe`. Use **Arquivo → Demonstração** (*File → Demo*) para ver uma árvore de exemplo, ou **Arquivo → Novo** (*File → New*) para começar a sua.
2. Mude para o modo **Edição** e adicione a primeira pessoa (*＋ New person*). Em uma árvore vazia, ela vira automaticamente o "eu".
3. Selecione uma pessoa e use **＋ Pai/mãe**, **＋ Parceiro(a)**, **＋ Filho(a)** ou **＋ Irmão/irmã** (*＋ Parent*, *＋ Partner*, *＋ Child*, *＋ Sibling*) para fazer a árvore crescer. Preencha nomes, datas, fotos e anotações à direita.
4. Escolha **Salvar como…** e dê um nome. A partir daí, as alterações são salvas automaticamente.
5. Mude para **Visualização** para explorar o mapa: arraste para mover, use a rolagem para dar zoom, **Ajustar** (*Fit to view*) para ver tudo e clique em um cartão para ver os detalhes.

## O arquivo `.famtree`

Um arquivo `.famtree` é um arquivo ZIP comum:

```
tree.json            people and relationships (plain, human-readable text)
media/<hash>.jpg     photos, resized to at most 1600 px
thumb/<hash>.jpg     small thumbnails for cards
```

- As fotos ficam dentro do arquivo, então você pode mandar um único `.famtree` para um parente sem perder nada. Fotos idênticas são armazenadas uma única vez.
- O `tree.json` pode ser lido em qualquer editor de texto e tem comentários descrevendo cada relacionamento. Se quiser editá-lo à mão, descompacte o arquivo, edite o `tree.json` e compacte de novo. Editá-lo diretamente dentro do `.famtree` vai corromper o arquivo.
- Arquivos antigos continuam abrindo nas versões mais novas do aplicativo.

## Limitações conhecidas

- As traduções da interface (exceto o russo) ainda não foram revisadas por falantes nativos. Correções são bem-vindas por meio de issues.
- As datas são sempre exibidas como `DD.MM.AAAA`, em qualquer idioma.
- Somente Windows 10.
- Ainda não há desfazer nem exportação para PNG/PDF.
- Importar muitas fotos de uma vez pode travar a janela por um instante.
- Fotos HEIC não são compatíveis (JPEG, PNG, WebP e BMP são).
- A miniatura de um cartão é um recorte centralizado, e ainda não dá para escolher a área do recorte.
- Em famílias muito grandes, o layout ainda pode posicionar pessoas em lugares estranhos ou cruzar linhas entre ramos (veja o roteiro).
- As nacionalidades não aparecem no modo Lista.

## Roteiro

Melhorias planejadas, sem ordem específica:

- [ ] **Um instalador de verdade**, em vez de um `.exe` solto do Actions, publicado em Releases.
- [ ] **Correções de bugs de layout**: de vez em quando as pessoas acabam no lugar errado do mapa.

## Compilando a partir do código-fonte

Se preferir compilar localmente, você precisa da toolchain do Rust no Windows:

```
cargo test --release
cargo build --release
```

O resultado é `target/release/famtree.exe`. O projeto usa [egui/eframe](https://github.com/emilk/egui) na interface.

## Relatando bugs

Encontrou um problema, principalmente na forma como as pessoas são posicionadas no mapa? Por favor, [abra uma issue](https://github.com/john3sobieski/family-tree/issues) e, se puder, anexe um pequeno arquivo `.famtree` que reproduza o problema (sem dados pessoais reais, se a árvore for privada).

## Licença

O FamTree é licenciado sob a [PolyForm Noncommercial License 1.0.0](https://polyformproject.org/licenses/noncommercial/1.0.0/). Veja o [`LICENSE.md`](LICENSE.md).

Em resumo:

- Você pode usar, copiar, modificar e compartilhar o aplicativo e o código-fonte para fins **não comerciais** (uso pessoal, história da família, hobby, educação, pesquisa e similares).
- **Vender ou usar comercialmente não é permitido.**
- Sempre que compartilhar o aplicativo ou uma versão modificada (um fork, uma recompilação, uma redistribuição), você deve manter os termos da licença e a linha `Required Notice` do `LICENSE.md`, que credita john3sobieski como autor.

Este resumo não é aconselhamento jurídico; o que vale é o texto da licença.
