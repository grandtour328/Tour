use windows::{
    core::*,
    Win32::System::Com::*,
    Win32::UI::Accessibility::*,
};

fn main() -> Result<()> {
    unsafe {
        // COM inicializálása
        CoInitializeEx(None, COINIT_APARTMENTTHREADED)?;

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

        // Érték lekérése
        let value = value_pattern.CurrentValue()?;
        println!("Elem értéke: {}", value.to_string_lossy());

        Ok(())
    }
}
