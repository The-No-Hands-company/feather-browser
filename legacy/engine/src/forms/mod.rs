// HTML Forms Module

#[derive(Debug, Clone)]
pub struct Form {
    pub action: String,
    pub method: FormMethod,
    pub enctype: FormEnctype,
    pub target: Option<String>,
    pub autocomplete: bool,
    pub novalidate: bool,
    pub elements: Vec<FormElement>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FormMethod {
    Get,
    Post,
    Dialog,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FormEnctype {
    UrlEncoded,
    Multipart,
    TextPlain,
}

#[derive(Debug, Clone)]
pub enum FormElement {
    Input(InputElement),
    TextArea(TextAreaElement),
    Select(SelectElement),
    Button(ButtonElement),
}

#[derive(Debug, Clone)]
pub struct InputElement {
    pub input_type: InputType,
    pub name: String,
    pub value: String,
    pub placeholder: Option<String>,
    pub required: bool,
    pub disabled: bool,
    pub readonly: bool,
    pub min: Option<String>,
    pub max: Option<String>,
    pub step: Option<String>,
    pub pattern: Option<String>,
    pub maxlength: Option<u32>,
    pub minlength: Option<u32>,
    pub autocomplete: Option<String>,
    pub checked: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputType {
    Text,
    Password,
    Email,
    Tel,
    Url,
    Search,
    Number,
    Range,
    Date,
    Time,
    DateTime,
    DateTimeLocal,
    Month,
    Week,
    Color,
    File,
    Checkbox,
    Radio,
    Submit,
    Reset,
    Button,
    Hidden,
}

#[derive(Debug, Clone)]
pub struct TextAreaElement {
    pub name: String,
    pub value: String,
    pub placeholder: Option<String>,
    pub rows: u32,
    pub cols: u32,
    pub required: bool,
    pub disabled: bool,
    pub readonly: bool,
    pub maxlength: Option<u32>,
    pub minlength: Option<u32>,
    pub wrap: TextWrap,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextWrap {
    Soft,
    Hard,
}

#[derive(Debug, Clone)]
pub struct SelectElement {
    pub name: String,
    pub options: Vec<SelectOption>,
    pub multiple: bool,
    pub required: bool,
    pub disabled: bool,
    pub size: u32,
}

#[derive(Debug, Clone)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
    pub selected: bool,
    pub disabled: bool,
}

#[derive(Debug, Clone)]
pub struct ButtonElement {
    pub button_type: ButtonType,
    pub name: Option<String>,
    pub value: Option<String>,
    pub disabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ButtonType {
    Submit,
    Reset,
    Button,
}

pub struct FormValidation {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
}

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub error_type: ValidationErrorType,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ValidationErrorType {
    Required,
    MinLength,
    MaxLength,
    Min,
    Max,
    Pattern,
    Type,
    Step,
}

impl Form {
    pub fn validate(&self) -> FormValidation {
        let mut errors = Vec::new();
        
        for element in &self.elements {
            match element {
                FormElement::Input(input) => {
                    if input.required && input.value.is_empty() {
                        errors.push(ValidationError {
                            field: input.name.clone(),
                            error_type: ValidationErrorType::Required,
                            message: "This field is required".to_string(),
                        });
                    }
                }
                FormElement::TextArea(textarea) => {
                    if textarea.required && textarea.value.is_empty() {
                        errors.push(ValidationError {
                            field: textarea.name.clone(),
                            error_type: ValidationErrorType::Required,
                            message: "This field is required".to_string(),
                        });
                    }
                }
                _ => {}
            }
        }
        
        FormValidation {
            valid: errors.is_empty(),
            errors,
        }
    }
}
