[English](README.md) | [简体中文](README.zh-CN.md) | [Español](README.es.md) | [Português (BR)](README.pt-BR.md) | [日本語](README.ja.md) | [Русский](README.ru.md) | **Français** | [Deutsch](README.de.md) | [한국어](README.ko.md) | [Türkçe](README.tr.md) | [Bahasa Indonesia](README.id.md)

# FamTree

Créé et maintenu par **[john3sobieski](https://github.com/john3sobieski)**. Dépôt : <https://github.com/john3sobieski/family-tree>

Une application de bureau pour Windows destinée à construire, modifier et parcourir votre arbre généalogique. Tout tient dans un seul fichier `.famtree` : les personnes, les liens de parenté et toutes les photos.

> **Statut :** développement précoce. Il n'existe pas encore de versions prêtes à l'emploi, et l'application n'est disponible qu'en la compilant via GitHub Actions (voir [Installation](#installation)). L'interface est pour l'instant en russe (voir [Feuille de route](#feuille-de-route)).

## Fonctionnalités

- **Trois modes**
  - **Consultation** (*Просмотр*) – carte mentale en lecture seule, avec zoom et déplacement fluides.
  - **Édition** (*Редакция*) – ajout et modification des personnes, des photos et des liens.
  - **Liste** (*Список*) – un tableau triable et filtrable de toutes les personnes de l'arbre.
- **Une carte mentale construite autour de vous.** Marquez une personne comme « moi » et l'arbre est dessiné de vos ancêtres les plus lointains vers le bas : parents, frères et sœurs, oncles, tantes, cousins et parents plus éloignés apparaissent donc automatiquement. Les deux côtés de la famille sont placés côte à côte, et les familles des alliés sont également incluses.
- **Cartes avec photo.** Chaque carte affiche une photo d'identité, le prénom, le nom de famille (avec le nom de jeune fille entre parenthèses pour les femmes), le patronyme, une bande de couleur pour le sexe et jusqu'à deux drapeaux de nationalité.
- **Fenêtre de la personne.** Cliquez sur une carte pour ouvrir une fenêtre plus grande avec la photo en grand, une galerie (plus de 10 photos par personne, aucun souci), les dates, l'âge, les lieux de naissance et de décès, la profession, des notes et des liens cliquables vers les parents, les partenaires et les enfants.
- **Recherche.** Saisissez un nom et la caméra file vers cette personne sur la carte et la met en surbrillance. En mode Liste, elle sert de filtre. `Ctrl+F` place le curseur dans le champ de recherche.
- **Liens de parenté.** Parents, partenaires, enfants et frères et sœurs peuvent être ajoutés en un clic ou rattachés à des personnes déjà existantes. Les cycles et les liens en double sont refusés.
- **Dates incomplètes.** Vous ne connaissez que l'année ? Aucun problème, n'importe quelle partie d'une date peut rester vide.
- **Nationalités.** Choisissez-en jusqu'à deux par personne dans une liste intégrée de 197 pays (tous les membres de l'ONU plus le Vatican, la Palestine, Taïwan et le Kosovo). Les drapeaux sont dessinés par l'application elle-même.
- **Conçu pour les grands arbres.** Seules les cartes visibles sont dessinées, les miniatures sont chargées à la demande (lazy loading), et la disposition n'est recalculée que lorsque la structure change. Testé avec des arbres de plusieurs milliers de personnes.
- **Sauvegarde automatique.** Une fois que l'arbre est associé à un fichier, chaque modification est enregistrée automatiquement un instant après que vous avez arrêté de saisir. Pour un nouvel arbre, utilisez d'abord **Enregistrer sous…** (*Сохранить как…*). L'application vous prévient aussi avant la fermeture si quelque chose n'a pas pu être enregistré.
- **Ouverture par double-clic.** Dans le menu **Fichier** (*Файл*), vous pouvez associer les fichiers `.famtree` à l'application.

## Installation

L'application ne fonctionne pour l'instant que sous Windows 10 et est compilée depuis les sources par GitHub Actions : vous n'avez donc besoin ni de Rust ni d'aucun autre outil sur votre machine. En revanche, il vous faut un compte GitHub.

1. Faites un **fork** du [dépôt](https://github.com/john3sobieski/family-tree) (ou utilisez votre propre copie).
2. Dans votre fork, ouvrez l'onglet **Actions**. Si GitHub vous le demande, activez les workflows pour le fork.
3. Sélectionnez le workflow **Build**, cliquez sur **Run workflow** et attendez la fin de l'exécution (environ 2 à 3 minutes). Il se lance aussi à chaque push.
4. Ouvrez l'exécution terminée et téléchargez l'artefact **`famtree-windows`** dans la section *Artifacts*, en bas de la page.
5. Décompressez-le. Il contient `famtree.exe`. Il n'y a rien à installer, il suffit de le lancer.

Remarques :

- Windows peut afficher un avertissement SmartScreen car le `.exe` n'est pas signé. Choisissez *Informations complémentaires → Exécuter quand même* si vous faites confiance à votre propre compilation.
- GitHub conserve les artefacts pendant une durée limitée (90 jours par défaut) : enregistrez donc le `.exe` dans un endroit permanent.
- Si vous déplacez `famtree.exe`, appuyez de nouveau sur le bouton d'association `.famtree` du menu **Fichier**, car l'association pointe vers l'emplacement de l'exécutable.

Lorsqu'un véritable installateur existera, il sera publié dans **Releases** et cette section sera remplacée.

## Démarrage rapide

1. Lancez `famtree.exe`. Utilisez **Fichier → Démo** (*Файл → Демо*) pour voir un arbre d'exemple, ou **Fichier → Nouveau** (*Файл → Новое*) pour commencer le vôtre.
2. Passez en mode **Édition** et ajoutez la première personne (*＋ Новый человек*). Dans un arbre vide, elle devient automatiquement « moi ».
3. Sélectionnez une personne et utilisez **＋ Parent**, **＋ Partenaire**, **＋ Enfant** ou **＋ Frère/sœur** (*＋ Родитель*, *＋ Партнёр*, *＋ Ребёнок*, *＋ Брат/сестра*) pour faire grandir l'arbre. Renseignez les noms, les dates, les photos et les notes sur la droite.
4. Choisissez **Enregistrer sous…** et donnez un nom. À partir de là, les modifications sont enregistrées automatiquement.
5. Passez en **Consultation** pour explorer la carte : faites glisser pour vous déplacer, utilisez la molette pour zoomer, **Ajuster** (*По размеру*) pour tout voir, et cliquez sur une carte pour afficher les détails.

## Le fichier `.famtree`

Un fichier `.famtree` est une archive ZIP ordinaire :

```
tree.json            people and relationships (plain, human-readable text)
media/<hash>.jpg     photos, resized to at most 1600 px
thumb/<hash>.jpg     small thumbnails for cards
```

- Les photos sont stockées dans le fichier : vous pouvez donc envoyer un seul `.famtree` à un proche sans rien perdre. Les photos identiques ne sont stockées qu'une seule fois.
- `tree.json` est lisible dans n'importe quel éditeur de texte et contient des commentaires décrivant chaque lien de parenté. Si vous voulez le modifier à la main, décompressez l'archive, modifiez le fichier, puis recompressez-le. Le modifier directement dans le `.famtree` corrompra l'archive.
- Les anciens fichiers continuent de s'ouvrir dans les nouvelles versions de l'application.

## Limitations connues

- L'interface n'est actuellement disponible qu'en russe.
- Windows 10 uniquement.
- Pas encore d'annulation ni d'export en PNG/PDF.
- L'importation d'un grand lot de photos d'un coup peut figer la fenêtre un instant.
- Les photos HEIC ne sont pas prises en charge (JPEG, PNG, WebP et BMP le sont).
- La miniature d'une carte est un recadrage centré, et vous ne pouvez pas encore choisir la zone recadrée.
- Dans les très grandes familles, la disposition peut encore placer des personnes à des endroits curieux ou faire se croiser des lignes entre branches (voir la feuille de route).
- Les nationalités ne sont pas affichées en mode Liste.

## Feuille de route

Améliorations prévues, sans ordre particulier :

- [ ] **Un véritable installateur**, à la place d'un simple `.exe` issu d'Actions, publié dans Releases.
- [ ] **Traduction de l'application** dans d'autres langues.
- [ ] **Traduction de ce README** dans d'autres langues.
- [ ] **Correction des bugs de disposition** : il arrive que des personnes se retrouvent au mauvais endroit sur la carte.

## Compiler depuis les sources

Si vous préférez compiler en local, il vous faut la toolchain Rust sous Windows :

```
cargo test --release
cargo build --release
```

Le résultat est `target/release/famtree.exe`. Le projet utilise [egui/eframe](https://github.com/emilk/egui) pour l'interface.

## Signalement de bugs

Vous avez trouvé un problème, en particulier dans la façon dont les personnes sont placées sur la carte ? [Ouvrez une issue](https://github.com/john3sobieski/family-tree/issues) et, si possible, joignez un petit fichier `.famtree` qui reproduit le problème (sans données personnelles réelles si l'arbre est privé).

## Licence

FamTree est distribué sous la [PolyForm Noncommercial License 1.0.0](https://polyformproject.org/licenses/noncommercial/1.0.0/). Voir [`LICENSE.md`](LICENSE.md).

En résumé :

- Vous pouvez utiliser, copier, modifier et partager l'application et son code source à des fins **non commerciales** (usage personnel, histoire familiale, loisir, enseignement, recherche, etc.).
- **La vente ou l'usage commercial n'est pas autorisé.**
- Chaque fois que vous partagez l'application ou une version modifiée (un fork, une recompilation, une redistribution), vous devez conserver les termes de la licence et la ligne `Required Notice` de `LICENSE.md`, qui désigne john3sobieski comme auteur.

Ce résumé ne constitue pas un avis juridique ; c'est le texte de la licence qui fait foi.
