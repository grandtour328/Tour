use windows::{
    core::*,
    Win32::System::Com::*,
    Win32::UI::Accessibility::*,
};

fn main() -> Result<()> {
    unsafe {
        // COM inicializálása – HRESULT-et ad vissza, ezért nem használható rajta a ?
        let hr = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        if hr.is_err() {
            println!("COM inicializálási hiba: {:?}", hr);
        }

        // UI Automation objektum létrehozása
        let uia: IUIAutomation = CoCreateInstance(
            &CUIAutomation,
            None,
            CLSCTX_INPROC_SERVER,
        )?;

        println!("UI Automation inicializálva.");

        // Fókuszált elem lekérése
        let element = uia.GetFocusedElement()?;
        println!("Fókuszált elem megvan.");

        // ValuePattern lekérése
        let value_pattern: IUIAutomationValuePattern =
            element.GetCurrentPatternAs(UIA_ValuePatternId)?;

        // Érték lekérése (BSTR → String)
        let value = value_pattern.CurrentValue()?;
        let value_str = value.to_string();

        println!("Elem értéke: {}", value_str);

        Ok(())
    }
}
