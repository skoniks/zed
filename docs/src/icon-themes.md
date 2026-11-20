---
title: Icon Themes
description: "Zed comes with a built-in icon theme, with more icon themes available as extensions."
---

# Icon Themes

Zed comes with a built-in icon theme, with more icon themes available as extensions.

## Selecting an Icon Theme

See what icon themes are installed and preview them via the Icon Theme Selector, which you can open from the command palette with {#action icon_theme_selector::Toggle}.

Use the arrow keys to preview a theme and `Enter` to save it. The Icon Theme Selector changes the shared theme for files and folders that don't have their own override.

## Installing more Icon Themes

More icon themes are available from the Extensions page, which you can access via the command palette with {#action zed::Extensions} or the [Zed website](https://zed.dev/extensions?filter=icon-themes).

## Configuring Icon Themes

Your selected icon theme is stored in your settings file.
You can open your settings file from the command palette with {#action zed::OpenSettingsFile} (bound to {#kb zed::OpenSettingsFile}).

Just like with themes, Zed allows for configuring different icon themes for light and dark mode.
You can set the mode to `"light"` or `"dark"` to ignore the current system mode.

```json [settings]
{
  "icon_theme": {
    "mode": "system",
    "light": "Light Icon Theme",
    "dark": "Dark Icon Theme"
  }
}
```

### Use different themes for files and folders

Install the icon theme you want, then run {#action icon_theme_selector::SelectFileIconTheme} or {#action icon_theme_selector::SelectFolderIconTheme} from the command palette. The folder selection also applies to folder chevrons.

Or add this to your `settings.json` to use Material Icon Theme for files and Zed's built-in icons for folders:

```json [settings]
{
  "file_icon_theme": "Material Icon Theme",
  "folder_icon_theme": "Zed (Default)"
}
```

Both overrides accept a theme name or a `mode` / `light` / `dark` object, just like `icon_theme`. To use the shared `icon_theme` again for either category, remove that override. The shared theme also applies when the override names a theme that isn't installed.

## Icon Theme Development

See: [Developing Zed Icon Themes](./extensions/icon-themes.md)
