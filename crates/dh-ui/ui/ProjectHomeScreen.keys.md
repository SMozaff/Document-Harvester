# ProjectHomeScreen — T1 key inventory

English is the current runtime catalogue for this isolated skeleton. Farsi is documented for T8; this component does not perform locale lookup or switch composition.

| Semantic key | English | Farsi | Direction / notes |
|---|---|---|---|
| `project.home.title` | New Project | پروژه جدید | Persian prose may be right-aligned. |
| `project.home.name.label` | Project name | نام پروژه | Persian prose; right-align in Farsi. |
| `project.home.name.placeholder` | Enter a project name | نام پروژه را وارد کنید | Persian prose; right-align in Farsi. |
| `project.home.parent.label` | Parent directory | پوشه والد | Persian prose; right-align in Farsi. |
| `project.home.parent.placeholder` | Choose where the project folder will be created | محل ایجاد پوشه پروژه را انتخاب کنید | Persian prose; right-align in Farsi. |
| `project.home.browse` | Browse… | مرور… | Action label; directional placement is shared. |
| `project.home.create` | Create Project | ایجاد پروژه | Action label; directional placement is shared. |
| `project.home.creating` | Creating… | در حال ایجاد… | Busy label; Persian prose may be right-aligned. |
| `project.home.last-project` | Last project | آخرین پروژه | Prefix for application-supplied project data. |
| `project.home.validation.name` | Enter a project name. | نام پروژه را وارد کنید. | Application-supplied reason; Persian prose. |
| `project.home.validation.parent` | Choose where the project folder will be created. | محل ایجاد پوشه پروژه را انتخاب کنید. | Application-supplied reason; Persian prose. |
| `project.home.validation.busy` | A project is being created. | پروژه‌ای در حال ایجاد است. | Application-supplied reason; Persian prose. |
| `project.home.error` | Application-supplied creation or recovery error. | خطای ایجاد یا بازیابی که برنامه ارائه می‌کند. | Do not translate inserted paths or identifiers. |

## Direction exceptions

`project-name` and all application-supplied error text are prose unless they contain technical data. `parent-directory` is a filesystem path and remains LTR in both locales. Project identifiers, extensions, page numbers, coordinates and other technical values remain LTR. No invisible bidi controls are inserted by this component.

## Keyboard and focus

The visual order is Project name → Parent directory → Browse → Create. Tab follows that order. Enter in the project-name field submits only when `create-enabled` is true and `busy` is false; the Create button uses the same guard. Escape is intentionally not consumed by this component: the application/window owns cancellation or dismissal, and no project is created by Escape. While busy, both fields, Browse and Create are disabled and duplicate submission is impossible. All actionable controls expose an accessible label and use at least the temporary 40 px target height.

## Interface contract

Inputs: `project-name` and `parent-directory` (`in-out string`), `create-enabled` (`bool`), `create-disabled-reason` (`string`), `busy` (`bool`), `error-text` (`string`), `last-project-info` (`string`), `text-alignment` (`string`, `left` or `right`), and `logo-placeholder-visible` (`bool`). Rust/dh-app owns all values and validation; the two form strings are observable two-way bindings for edit reporting.

Callbacks: `choose-parent-directory()` and `create-project(string name, string parent-directory)` are intents only. `project-form-changed(string name, string parent-directory)` reports edits so dh-app can recompute availability. There is no `open-last-project()` callback because the current application restores the last project automatically at startup and exposes no separate user action.

Minimum size: 720×520 at the window level. Preferred screen content: 520×360. The form keeps the primary action in normal flow; long paths remain discoverable through the read-only LineEdit's horizontal scrolling, and long messages wrap in bounded content. The named `logo-placeholder` slot is off by default and does not substitute for canonical assets.

## Deferred MainWindow integration patch

Apply this in the owning shell branch, replacing the current `if !root.project-open: ScrollView { ... }` project-home block. This is intentionally not applied by T1:

```slint
if !root.project-open: ProjectHomeScreen {
    project-name <=> root.project-name;
    parent-directory <=> root.parent-directory;
    create-enabled: root.create-project-enabled;
    create-disabled-reason: root.create-project-note;
    busy: root.project-busy;
    error-text: root.project-error-text;
    text-alignment: "left";
    choose-parent-directory => { root.choose-parent-directory(); }
    project-form-changed(name, parent) => {
        root.project-form-changed(name, parent);
    }
    create-project(name, parent) => {
        root.create-project(name, parent);
    }
}
```

The shell owner must add the Rust-owned `project-error-text` property (or map an equivalent operation-scoped error input) because the current `status-message` also carries success and information messages and must not be rendered as an error. The shell owner also supplies `last-project-info` if a distinct app-owned recovery label is desired and passes `text-alignment: "right"` after T8 resolves Persian prose alignment. Existing `on_project_form_changed`, `on_choose_parent_directory` and `on_create_project` handlers in `crates/dh-app/src/main.rs` are the intended callback targets.
