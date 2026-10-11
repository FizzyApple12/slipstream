# Third-party notices

Third-party code and assets retain their own copyright notices and license
terms. The project's GPL license grant does not replace those terms or grant
rights to material the project does not own. These notices cover the material
identified below; they are not a complete inventory of dependency licenses.

## Signalsmith Audio

[`libdsp/src/lib.rs`](libdsp/src/lib.rs) acknowledges Signalsmith Linear,
Signalsmith DSP, and Signalsmith Stretch as references. The implementation also
contains a `SignalsmithStretch` type and comments referring to translations of
the original implementations. The following upstream MIT notices are retained
for Signalsmith code and adaptations in `libdsp`:

| Project | Copyright notice | Included license |
| --- | --- | --- |
| [Signalsmith Linear](https://github.com/Signalsmith-Audio/linear) | Copyright (c) 2025 Signalsmith Audio | [MIT](LICENSES/Signalsmith-Linear-MIT.txt) |
| [Signalsmith DSP](https://github.com/Signalsmith-Audio/dsp) | Copyright (c) 2021 Geraint Luff / Signalsmith Audio Ltd. | [MIT](LICENSES/Signalsmith-DSP-MIT.txt) |
| [Signalsmith Stretch](https://github.com/Signalsmith-Audio/signalsmith-stretch) | Copyright (c) 2022 Geraint Luff / Signalsmith Audio Ltd. | [MIT](LICENSES/Signalsmith-Stretch-MIT.txt) |

The license texts were copied verbatim from these upstream revisions:

- [Linear: `146b26f91b48835d665fa0498d64d9617818d27a`](https://github.com/Signalsmith-Audio/linear/blob/146b26f91b48835d665fa0498d64d9617818d27a/LICENSE.txt)
- [DSP: `4f62b0a8783c483c353d0232654fe0ffea3cd434`](https://github.com/Signalsmith-Audio/dsp/blob/4f62b0a8783c483c353d0232654fe0ffea3cd434/LICENSE.txt)
- [Stretch: `a670068d9aeb64913331d5cc29337b19a457a7df`](https://github.com/Signalsmith-Audio/signalsmith-stretch/blob/a670068d9aeb64913331d5cc29337b19a457a7df/LICENSE.txt)

These revisions identify the license text snapshots, not the versions originally
used as references or adapted into Slipstream.

## Lucide icons

SVG metadata identifies the following files in
[`slipstream-embedded/qml/icons/`](slipstream-embedded/qml/icons/) as Lucide icons
or adaptations:

```text
album.svg              artist.svg             caps-filled-dark.svg
caps-filled.svg        check.svg              chevron-left.svg
chevron-right.svg      delete.svg             eject.svg
empty.svg              enter.svg              error.svg
folder.svg             globe.svg              info.svg
key.svg                loader.svg             network.svg
nothing.svg            player.svg             playlist.svg
search.svg             settings.svg           shift-filled-dark.svg
shift-filled.svg       shift.svg              space.svg
track.svg              usb.svg                warning.svg
```

Lucide uses the ISC license, with MIT notices for icons derived from Feather.
The full upstream notice, including the Feather icon list and both licenses,
is preserved in [LICENSES/Lucide-ISC-MIT.txt](LICENSES/Lucide-ISC-MIT.txt).

That text was copied verbatim from
[Lucide revision `70562c1ee1c4fdcf736fe97bc893fb8511927934`](https://github.com/lucide-icons/lucide/blob/70562c1ee1c4fdcf736fe97bc893fb8511927934/LICENSE).
This is a license text snapshot, not a claim about the original icon download
version. See also the [Lucide license page](https://lucide.dev/license).

## Website fonts

The self-hosted Latin subsets in [`website/assets/fonts/`](website/assets/fonts/)
retain the SIL Open Font License 1.1. The font files are not relicensed under
the project's GPL grant.

| Font file | Copyright notice | Included license |
| --- | --- | --- |
| `barlow-condensed-latin-800.woff2` | Copyright 2017 The Barlow Project Authors | [Barlow Condensed OFL](website/assets/fonts/Barlow-Condensed-OFL.txt) |
| `dm-sans-latin-400-700.woff2` | Copyright 2014 The DM Sans Project Authors | [DM Sans OFL](website/assets/fonts/DM-Sans-OFL.txt) |
| `ibm-plex-mono-latin-400.woff2` | Copyright © 2017 IBM Corp., Reserved Font Name "Plex" | [IBM Plex Mono OFL](website/assets/fonts/IBM-Plex-Mono-OFL.txt) |

The Google Fonts download URLs are recorded beside each font declaration in
[`fonts.css`](website/assets/fonts/fonts.css). Keep the associated OFL notices
with redistributed copies of these fonts.

## Embedded Helvetica font: separate rights required

[`slipstream-embedded/qml/fonts/helvetica.ttf`](slipstream-embedded/qml/fonts/helvetica.ttf)
is **excluded from the project's GPL license grant**. Its embedded metadata
identifies Monotype Imaging Inc. as copyright holder, with a 2019–2021 copyright
notice. Its embedded license description says copying and distribution are not
permitted unless a separate agreement grants additional rights.

The file is currently bundled by
[`fonts.qrc`](slipstream-embedded/qml/fonts/fonts.qrc). This repository does not
establish permission to redistribute it. Excluding the font from the project
license does not provide that permission. Verify the applicable redistribution
rights or replace the font with an appropriately licensed alternative before
redistributing it, including in an application build.

## Selected dependencies

- **Qt:** Each module retains its Qt license. The current application links
  Qt Virtual Keyboard, which is available under GPLv3 or a commercial Qt
  license. A combined application using its open-source edition must satisfy
  GPLv3; the project's GPL-2.0-or-later grant permits that choice. See
  [Qt Virtual Keyboard licensing](https://doc.qt.io/qt-6/qtvirtualkeyboard-index.html#licenses-and-attributions)
  and [Qt licensing](https://doc.qt.io/qt-6/licensing.html).
- **rekordcrate:** [`libdatabase/Cargo.toml`](libdatabase/Cargo.toml) uses the
  FizzyApple12 fork. [`Cargo.lock`](Cargo.lock) pins revision
  `2d06d7e872ef36e7e2920d3b057a1a8d5bf55439`, whose
  [manifest declares MPL-2.0](https://github.com/FizzyApple12/rekordcrate/blob/2d06d7e872ef36e7e2920d3b057a1a8d5bf55439/Cargo.toml).
  Its [COPYING file](https://github.com/FizzyApple12/rekordcrate/blob/2d06d7e872ef36e7e2920d3b057a1a8d5bf55439/COPYING)
  and source notices govern that dependency. Preserve its notices and provide
  recipients access to its covered source when distributing compiled copies.

Other Cargo and native dependencies retain their respective upstream licenses.
Include their required notices and source disclosures when distributing them.
