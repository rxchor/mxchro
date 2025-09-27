use indexmap::IndexMap;
use windows::Win32::UI::Input::KeyboardAndMouse::*;

#[allow(dead_code)]
pub struct KeyMapper {
    str_to_vk: IndexMap<String, u16>,
    vk_to_str: IndexMap<u16, String>,
}

#[allow(dead_code)]
impl KeyMapper {

    // No singleton pattern due to personal choice
    // I prefer to release memory and create a new one if needed
    pub fn new() -> Self {
        let mut key_mapper = KeyMapper {
            str_to_vk: IndexMap::new(),
            vk_to_str: IndexMap::new()
        };

        // VK_KEYS gotten from windows api:
        // reference: https://learn.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes

        // inputs are hard coded because keyboard layout is most likely not going to change
        // and the VK mapping methods the windows api has (such as MapVirtualKeyA)
        // only have support for characters and not special characters (like F2, F3, Enter, etc...)
        
        // letters A-Z
        for i in 0..26 {
        let c = ((b'A' + i) as char).to_string();
            key_mapper.insert(c.clone(), (0x41 + i) as u16);
        }

        // digits 0-9
        for i in 0..=9 {
        let key = i.to_string();
            key_mapper.insert(key.clone(), (0x30 + i) as u16);
        }

        // Funciones F1-F24
        for i in 1..=24 {
        let key = format!("F{}", i);
            key_mapper.insert(key, (0x70 + (i - 1)) as u16);
        }

        // Mouse buttons
        key_mapper.insert("mouse_left".to_string(), VK_LBUTTON.0);
        key_mapper.insert("mouse_right".to_string(), VK_RBUTTON.0);
        key_mapper.insert("mouse_middle".to_string(), VK_MBUTTON.0);
        key_mapper.insert("mouse_x1".to_string(), VK_XBUTTON1.0);
        key_mapper.insert("mouse_x2".to_string(), VK_XBUTTON2.0);

        // Basic keys
        key_mapper.insert("backspace".to_string(), VK_BACK.0);
        key_mapper.insert("tab".to_string(), VK_TAB.0);
        key_mapper.insert("clear".to_string(), VK_CLEAR.0);
        key_mapper.insert("enter".to_string(), VK_RETURN.0);
        key_mapper.insert("shift".to_string(), VK_SHIFT.0);
        key_mapper.insert("ctrl".to_string(), VK_CONTROL.0);
        key_mapper.insert("alt".to_string(), VK_MENU.0);
        key_mapper.insert("pause".to_string(), VK_PAUSE.0);
        key_mapper.insert("capslock".to_string(), VK_CAPITAL.0);
        key_mapper.insert("esc".to_string(), VK_ESCAPE.0);
        key_mapper.insert("space".to_string(), VK_SPACE.0);

        // Navigation
        key_mapper.insert("pageup".to_string(), VK_PRIOR.0);
        key_mapper.insert("pagedown".to_string(), VK_NEXT.0);
        key_mapper.insert("end".to_string(), VK_END.0);
        key_mapper.insert("home".to_string(), VK_HOME.0);
        key_mapper.insert("left".to_string(), VK_LEFT.0);
        key_mapper.insert("up".to_string(), VK_UP.0);
        key_mapper.insert("right".to_string(), VK_RIGHT.0);
        key_mapper.insert("down".to_string(), VK_DOWN.0);
        key_mapper.insert("insert".to_string(), VK_INSERT.0);
        key_mapper.insert("delete".to_string(), VK_DELETE.0);


        // Numpad
        for i in 0..=9 {
        let key = format!("numpad{}", i);
            key_mapper.insert(key.clone(), (0x60 + i) as u16);
        }
        key_mapper.insert("numpad_mul".to_string(), VK_MULTIPLY.0);
        key_mapper.insert("numpad_add".to_string(), VK_ADD.0);
        key_mapper.insert("numpad_sep".to_string(), VK_SEPARATOR.0);
        key_mapper.insert("numpad_sub".to_string(), VK_SUBTRACT.0);
        key_mapper.insert("numpad_decimal".to_string(), VK_DECIMAL.0);
        key_mapper.insert("numpad_div".to_string(), VK_DIVIDE.0);


        // Special Keys
        key_mapper.insert("win_left".to_string(), VK_LWIN.0);
        key_mapper.insert("win_right".to_string(), VK_RWIN.0);
        key_mapper.insert("apps".to_string(), VK_APPS.0);
        key_mapper.insert("numlock".to_string(), VK_NUMLOCK.0);
        key_mapper.insert("scrolllock".to_string(), VK_SCROLL.0);


        // Media keys
        key_mapper.insert("vol_mute".to_string(), VK_VOLUME_MUTE.0);
        key_mapper.insert("vol_down".to_string(), VK_VOLUME_DOWN.0);
        key_mapper.insert("vol_up".to_string(), VK_VOLUME_UP.0);
        key_mapper.insert("media_next".to_string(), VK_MEDIA_NEXT_TRACK.0);
        key_mapper.insert("media_prev".to_string(), VK_MEDIA_PREV_TRACK.0);
        key_mapper.insert("'".to_string(), VK_OEM_7.0);

        return key_mapper;
    }

    fn insert(&mut self, str: String, vk: u16) {
        if !self.vk_to_str.contains_key(&vk) && !self.str_to_vk.contains_key(&str) {
            self.vk_to_str.insert(vk, str.clone());
            self.str_to_vk.insert(str, vk);
        }
    }

    pub fn vk_to_string(&self, vk: &u16) -> Option<String> {
        let string = self.vk_to_str.get(vk);

        if string.is_none() {
            return None;
        }

        return Some(string.unwrap().clone());
    }

    pub fn string_to_vk(&self, string: &String) -> Option<u16> {
        let vk_code = self.str_to_vk.get(string);

        if vk_code.is_none() {
            return None;
        }

        return Some(vk_code.unwrap().clone());
    }

    pub fn strings_vec(&self) -> Vec<String> {
        return self.vk_to_str.values().into_iter().map(|s| s.clone()).collect();
    }

    pub fn vk_vec(&self) -> Vec<u16> {
        return self.str_to_vk.values().into_iter().map(|vk| vk.clone()).collect();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vk_to_string_basic_keys() {
        let mapper = KeyMapper::new();
        assert_eq!(mapper.vk_to_string(&VK_A.0), Some("A".to_string())); // 'A'
        assert_eq!(mapper.vk_to_string(&VK_0.0), Some("0".to_string())); // '0'
        assert_eq!(mapper.vk_to_string(&VK_RETURN.0), Some("enter".to_string()));
        assert_eq!(mapper.vk_to_string(&VK_LBUTTON.0), Some("mouse_left".to_string()));
    }

    #[test]
    fn test_string_to_vk_basic_keys() {
        let mapper = KeyMapper::new();
        assert_eq!(mapper.string_to_vk(&"A".to_string()), Some(VK_A.0));
        assert_eq!(mapper.string_to_vk(&"0".to_string()), Some(VK_0.0));
        assert_eq!(mapper.string_to_vk(&"enter".to_string()), Some(VK_RETURN.0));
        assert_eq!(mapper.string_to_vk(&"mouse_left".to_string()), Some(VK_LBUTTON.0));
    }

    #[test]
    fn test_vk_to_string_not_found() {
        let mapper = KeyMapper::new();
        assert_eq!(mapper.vk_to_string(&9999), None);
    }

    #[test]
    fn test_string_to_vk_not_found() {
        let mapper = KeyMapper::new();
        assert_eq!(mapper.string_to_vk(&"not_a_key".to_string()), None);
    }

    #[test]
    fn test_vk_to_string_and_back() {
        let mapper = KeyMapper::new();
        let keys = vec!["A", "enter", "mouse_left", "F12", "numpad5"];
        for key in keys {
            let vk = mapper.string_to_vk(&key.to_string()).unwrap();
            let key_back = mapper.vk_to_string(&vk).unwrap();
            assert_eq!(key, key_back);
        }
    }
}