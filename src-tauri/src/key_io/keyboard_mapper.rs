use bimap::BiMap;
use windows::Win32::UI::Input::KeyboardAndMouse::*;

#[allow(dead_code)]
pub struct KeyboardMapper {
    vk_string: BiMap<String, u16>
}

#[allow(dead_code)]
impl KeyboardMapper {

    // No singleton pattern due to personal choice
    // I prefer to release memory and create a new one if needed
    pub fn new() -> Self {
        let mut  vk_str = BiMap::new();

        // VK_KEYS gotten from windows api:
        // reference: https://learn.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes

        // inputs are hard coded because keyboard layout is most likely not going to change
        // and the VK mapping methods the windows api has (such as MapVirtualKeyA)
        // only have support for characters and not special characters (like F2, F3, Enter, etc...)
        
        // Mouse buttons
        vk_str.insert("mouse_left".to_string(), VK_LBUTTON.0);
        vk_str.insert("mouse_right".to_string(), VK_RBUTTON.0);
        vk_str.insert("mouse_middle".to_string(), VK_MBUTTON.0);
        vk_str.insert("mouse_x1".to_string(), VK_XBUTTON1.0);
        vk_str.insert("mouse_x2".to_string(), VK_XBUTTON2.0);

        // Basic keys
        vk_str.insert("backspace".to_string(), VK_BACK.0);
        vk_str.insert("tab".to_string(), VK_TAB.0);
        vk_str.insert("clear".to_string(), VK_CLEAR.0);
        vk_str.insert("enter".to_string(), VK_RETURN.0);
        vk_str.insert("shift".to_string(), VK_SHIFT.0);
        vk_str.insert("ctrl".to_string(), VK_CONTROL.0);
        vk_str.insert("alt".to_string(), VK_MENU.0);
        vk_str.insert("pause".to_string(), VK_PAUSE.0);
        vk_str.insert("capslock".to_string(), VK_CAPITAL.0);
        vk_str.insert("esc".to_string(), VK_ESCAPE.0);
        vk_str.insert("space".to_string(), VK_SPACE.0);

        // Navigation
        vk_str.insert("pageup".to_string(), VK_PRIOR.0);
        vk_str.insert("pagedown".to_string(), VK_NEXT.0);
        vk_str.insert("end".to_string(), VK_END.0);
        vk_str.insert("home".to_string(), VK_HOME.0);
        vk_str.insert("left".to_string(), VK_LEFT.0);
        vk_str.insert("up".to_string(), VK_UP.0);
        vk_str.insert("right".to_string(), VK_RIGHT.0);
        vk_str.insert("down".to_string(), VK_DOWN.0);
        vk_str.insert("insert".to_string(), VK_INSERT.0);
        vk_str.insert("delete".to_string(), VK_DELETE.0);


        // digits 0-9
        for i in 0..=9 {
        let key = i.to_string();
            vk_str.insert(key.clone(), (0x30 + i) as u16);
        }


        // letters A-Z
        for i in 0..26 {
        let c = ((b'a' + i) as char).to_string();
            vk_str.insert(c.clone(), (0x41 + i) as u16);
        }


        // Funciones F1-F24
        for i in 1..=24 {
        let key = format!("f{}", i);
            vk_str.insert(key, (0x70 + (i - 1)) as u16);
        }


        // Numpad
        for i in 0..=9 {
        let key = format!("numpad{}", i);
            vk_str.insert(key.clone(), (0x60 + i) as u16);
        }
        vk_str.insert("numpad_mul".to_string(), VK_MULTIPLY.0);
        vk_str.insert("numpad_add".to_string(), VK_ADD.0);
        vk_str.insert("numpad_sep".to_string(), VK_SEPARATOR.0);
        vk_str.insert("numpad_sub".to_string(), VK_SUBTRACT.0);
        vk_str.insert("numpad_decimal".to_string(), VK_DECIMAL.0);
        vk_str.insert("numpad_div".to_string(), VK_DIVIDE.0);


        // Special Keys
        vk_str.insert("win_left".to_string(), VK_LWIN.0);
        vk_str.insert("win_right".to_string(), VK_RWIN.0);
        vk_str.insert("apps".to_string(), VK_APPS.0);
        vk_str.insert("numlock".to_string(), VK_NUMLOCK.0);
        vk_str.insert("scrolllock".to_string(), VK_SCROLL.0);


        // Media keys
        vk_str.insert("vol_mute".to_string(), VK_VOLUME_MUTE.0);
        vk_str.insert("vol_down".to_string(), VK_VOLUME_DOWN.0);
        vk_str.insert("vol_up".to_string(), VK_VOLUME_UP.0);
        vk_str.insert("media_next".to_string(), VK_MEDIA_NEXT_TRACK.0);
        vk_str.insert("media_prev".to_string(), VK_MEDIA_PREV_TRACK.0);
        vk_str.insert("'".to_string(), VK_OEM_7.0);

        KeyboardMapper { vk_string: vk_str }
    }

    pub fn vk_to_string(&self, vk: &u16) -> Option<String> {
        let string = self.vk_string.get_by_right(vk);

        if string.is_none() {
            return None;
        }

        return Some(string.unwrap().clone());
    }

    pub fn string_to_vk(&self, string: &String) -> Option<u16> {
        let vk_code = self.vk_string.get_by_left(string);

        if vk_code.is_none() {
            return None;
        }

        return Some(vk_code.unwrap().clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vk_to_string_basic_keys() {
        let mapper = KeyboardMapper::new();
        assert_eq!(mapper.vk_to_string(&VK_A.0), Some("a".to_string())); // 'A'
        assert_eq!(mapper.vk_to_string(&VK_0.0), Some("0".to_string())); // '0'
        assert_eq!(mapper.vk_to_string(&VK_RETURN.0), Some("enter".to_string()));
        assert_eq!(mapper.vk_to_string(&VK_LBUTTON.0), Some("mouse_left".to_string()));
    }

    #[test]
    fn test_string_to_vk_basic_keys() {
        let mapper = KeyboardMapper::new();
        assert_eq!(mapper.string_to_vk(&"a".to_string()), Some(0x41));
        assert_eq!(mapper.string_to_vk(&"0".to_string()), Some(0x30));
        assert_eq!(mapper.string_to_vk(&"enter".to_string()), Some(VK_RETURN.0));
        assert_eq!(mapper.string_to_vk(&"mouse_left".to_string()), Some(VK_LBUTTON.0));
    }

    #[test]
    fn test_vk_to_string_not_found() {
        let mapper = KeyboardMapper::new();
        assert_eq!(mapper.vk_to_string(&9999), None);
    }

    #[test]
    fn test_string_to_vk_not_found() {
        let mapper = KeyboardMapper::new();
        assert_eq!(mapper.string_to_vk(&"not_a_key".to_string()), None);
    }

    #[test]
    fn test_vk_to_string_and_back() {
        let mapper = KeyboardMapper::new();
        let keys = vec!["a", "enter", "mouse_left", "f12", "numpad5"];
        for key in keys {
            let vk = mapper.string_to_vk(&key.to_string()).unwrap();
            let key_back = mapper.vk_to_string(&vk).unwrap();
            assert_eq!(key, key_back);
        }
    }
}