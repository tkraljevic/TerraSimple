//! Prilagodba unosa sustavu. Na Macu je ⌃ + klik desni klik, kao u ostalim programima za Mac.
//! Na Windowsu i Linuxu Ctrl + klik ostaje običan klik.

use eframe::egui::{Event, PointerButton, RawInput, os::OperatingSystem};

/// Pamti je li pritisak bio ⌃ + klik, da i otpuštanje bude desno, iako je ⌃ možda već pušten.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DesniKlikMaca {
    pritisnut: bool,
}

impl DesniKlikMaca {
    pub fn obradi(&mut self, sustav: OperatingSystem, unos: &mut RawInput) {
        if sustav != OperatingSystem::Mac {
            return;
        }
        for dogadaj in &mut unos.events {
            self.dogadaj(dogadaj);
        }
    }

    fn dogadaj(&mut self, dogadaj: &mut Event) {
        let Event::PointerButton {
            button: tipka @ PointerButton::Primary,
            pressed,
            modifiers,
            ..
        } = dogadaj
        else {
            return;
        };
        let desni = if *pressed {
            modifiers.ctrl && !modifiers.mac_cmd
        } else {
            self.pritisnut
        };
        self.pritisnut = *pressed && desni;
        if desni {
            *tipka = PointerButton::Secondary;
            modifiers.ctrl = false;
        }
    }
}

#[cfg(test)]
mod testovi {
    use super::*;
    use eframe::egui::{Modifiers, pos2};

    fn klik(pressed: bool, modifiers: Modifiers) -> Event {
        Event::PointerButton {
            pos: pos2(1.0, 1.0),
            button: PointerButton::Primary,
            pressed,
            modifiers,
        }
    }

    fn tipke(sustav: OperatingSystem, dogadaji: Vec<Event>) -> Vec<PointerButton> {
        let mut unos = RawInput {
            events: dogadaji,
            ..RawInput::default()
        };
        DesniKlikMaca::default().obradi(sustav, &mut unos);
        unos.events
            .iter()
            .filter_map(|d| match d {
                Event::PointerButton { button, .. } => Some(*button),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn ctrl_klik_je_desni_samo_na_macu() {
        let ctrl = Modifiers::CTRL;
        // ⌃ se pusti prije tipke miša, a otpuštanje je i dalje desno.
        let dogadaji = vec![
            klik(true, ctrl),
            klik(false, Modifiers::NONE),
            Event::PointerMoved(pos2(2.0, 2.0)),
            klik(true, Modifiers::NONE),
            klik(false, Modifiers::NONE),
        ];
        assert_eq!(
            tipke(OperatingSystem::Mac, dogadaji.clone()),
            [
                PointerButton::Secondary,
                PointerButton::Secondary,
                PointerButton::Primary,
                PointerButton::Primary
            ]
        );
        assert!(
            tipke(OperatingSystem::Windows, dogadaji)
                .iter()
                .all(|t| *t == PointerButton::Primary)
        );
        // ⌘ + klik nije desni klik.
        let cmd = Modifiers {
            ctrl: true,
            mac_cmd: true,
            ..Modifiers::NONE
        };
        assert_eq!(
            tipke(OperatingSystem::Mac, vec![klik(true, cmd)]),
            [PointerButton::Primary]
        );
    }

    #[test]
    fn desna_tipka_ostaje_desna() {
        let mut d = Event::PointerButton {
            pos: pos2(1.0, 1.0),
            button: PointerButton::Secondary,
            pressed: true,
            modifiers: Modifiers::CTRL,
        };
        let mut stanje = DesniKlikMaca::default();
        stanje.dogadaj(&mut d);
        assert_eq!(stanje, DesniKlikMaca::default());
    }
}
