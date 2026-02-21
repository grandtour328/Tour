use windows::{
    core::*,
    Win32::UI::Accessibility::*,
    Win32::Foundation::*,
};

fn main() -> Result<()> {
    unsafe {
        // UI Automation inicializálása
        let uia: IUIAutomation = CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER)?;

        // Fókuszált elem lekérése
        let element = uia.GetFocusedElement()?;

        println!("Fókuszált elem megvan.");

        // A ValuePattern lekérése
        let value_pattern: IUIAutomationValuePattern = element.GetCurrentPatternAs(UIA_ValuePatternId)?;

        // Az aktuális érték lekérése
        let value = value_pattern.CurrentValue()?;

        println!("Elem értéke: {}", value.to_string_lossy());

        Ok(())
    }
}
