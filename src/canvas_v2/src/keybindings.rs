use winit::event::ModifiersState;
use winit::keyboard::KeyCode;

pub struct Keybinding {
    pub key: KeyCode,
    pub modifiers: ModifiersState,
    pub action: Action,
}

pub enum Action {
    // Text editing
    InsertNewline,
    DeleteWordBackward,
    DeleteWordForward,
    MoveWordLeft,
    MoveWordRight,
    MoveLineStart,
    MoveLineEnd,
    SelectAll,
    
    // Navigation
    MoveUp,
    MoveDown,
    PageUp,
    PageDown,
    
    // Commands
    Submit,
    Cancel,
    
    // System
    Quit,
}

impl Keybinding {
    pub fn mac_defaults() -> Vec<Self> {
        vec![
            // Text editing - Mac style
            Keybinding {
                key: KeyCode::Enter,
                modifiers: ModifiersState::SHIFT,
                action: Action::InsertNewline,
            },
            Keybinding {
                key: KeyCode::Backspace,
                modifiers: ModifiersState::ALT,
                action: Action::DeleteWordBackward,
            },
            Keybinding {
                key: KeyCode::Delete,
                modifiers: ModifiersState::ALT,
                action: Action::DeleteWordForward,
            },
            Keybinding {
                key: KeyCode::ArrowLeft,
                modifiers: ModifiersState::ALT,
                action: Action::MoveWordLeft,
            },
            Keybinding {
                key: KeyCode::ArrowRight,
                modifiers: ModifiersState::ALT,
                action: Action::MoveWordRight,
            },
            Keybinding {
                key: KeyCode::ArrowLeft,
                modifiers: ModifiersState::SUPER,
                action: Action::MoveLineStart,
            },
            Keybinding {
                key: KeyCode::ArrowRight,
                modifiers: ModifiersState::SUPER,
                action: Action::MoveLineEnd,
            },
            Keybinding {
                key: KeyCode::KeyA,
                modifiers: ModifiersState::SUPER,
                action: Action::SelectAll,
            },
            
            // Navigation
            Keybinding {
                key: KeyCode::ArrowUp,
                modifiers: ModifiersState::empty(),
                action: Action::MoveUp,
            },
            Keybinding {
                key: KeyCode::ArrowDown,
                modifiers: ModifiersState::empty(),
                action: Action::MoveDown,
            },
            
            // Commands
            Keybinding {
                key: KeyCode::Enter,
                modifiers: ModifiersState::empty(),
                action: Action::Submit,
            },
            Keybinding {
                key: KeyCode::Escape,
                modifiers: ModifiersState::empty(),
                action: Action::Cancel,
            },
            
            // System
            Keybinding {
                key: KeyCode::KeyQ,
                modifiers: ModifiersState::SUPER,
                action: Action::Quit,
            },
        ]
    }
}