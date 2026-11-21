"""Regression tests for source-locked draft Hebrew TextureCraft catalog."""
import sys
import unittest
import unicodedata
from pathlib import Path

ROOT=Path(__file__).resolve().parent.parent
sys.path.insert(0,str(ROOT/'tools'))
import sync_app
from export_texturecraft import resolved

class HebrewTextureCraftTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.app, cls.parse_errors=sync_app.parse((ROOT/'locales/he/texturecraft.tsv').read_text('utf-8'))
        cls.common, cls.common_errors=sync_app.parse((ROOT/'locales/he/common.tsv').read_text('utf-8'))
        cls.source, cls.source_errors=sync_app.parse((ROOT/'locales/ar/texturecraft.tsv').read_text('utf-8'))
        cls.additional_source_keys=set()
        for language in ('de','es','fr','ja','ko'):
            source, errors=sync_app.parse((ROOT/f'locales/{language}/texturecraft.tsv').read_text('utf-8'))
            assert not errors, (language, errors)
            cls.additional_source_keys.update(source)

    def test_hebrew_app_catalog_includes_pbr_short_label_batch(self):
        self.assertFalse(self.parse_errors)
        self.assertGreaterEqual(len(self.app),958)
        self.assertTrue(all(c=='' for c,s in self.app))
        self.assertTrue(all(s and v for (c,s),v in self.app.items()))

    def test_round6_74_additions_are_source_backed_and_resolve(self):
        locale=resolved(ROOT,'he-IL')
        examples={
            'Depth of field':'עומק שדה',
            'Cores kept for the UI':'ליבות שמורות לממשק',
            'Interface scale':'קנה מידה של הממשק',
            'Environment exposure (EV)':'חשיפת הסביבה (EV)',
            'Scatter radius':'רדיוס פיזור',
            'Index of refraction':'מקדם שבירה',
            'Quarter':'רבע',
            'Start window':'חלון פתיחה',
        }
        for english,hebrew in examples.items():
            self.assertEqual(locale[('',english)],hebrew,english)
            self.assertIsNone(sync_app.text_problem('',english,hebrew))

    def test_round7_material_and_rendering_english_keys(self):
        locale=resolved(ROOT,'he-IL')
        examples={
            'Cube': 'קובייה',
            'MaterialX materials': 'חומרי MaterialX',
            'No Parameters': 'ללא פרמטרים',
            'Roughness': 'חספוס',
            'My brushes': 'המברשות שלי',
            'Height offset': 'היסט גובה',
            'Radial symmetry': 'סימטריה רדיאלית',
            'Triangle fill': 'מילוי משולש',
        }
        for english,hebrew in examples.items():
            self.assertEqual(locale[('',english)],hebrew,english)
            self.assertIsNone(sync_app.text_problem('',english,hebrew))

    def test_round8_rendering_and_materials_english_keys(self):
        locale=resolved(ROOT,'he-IL')
        examples={
            'Clamp': 'הגבלת טווח',
            'Vivid light': 'אור חי',
            'Histogram Scan': 'סריקת היסטוגרמה',
            'Place as decal': 'מיקום כמדבקה',
            'Relative to Bounding Box': 'יחסי לתיבה התוחמת',
            'Visible polygons only': 'מצולעים גלויים בלבד',
        }
        for english,hebrew in examples.items():
            self.assertEqual(locale[('',english)],hebrew,english)
            self.assertIsNone(sync_app.text_problem('',english,hebrew))

    def test_new_path_painting_and_rendering_tooltips(self):
        locale=resolved(ROOT,'he-IL')
        samples={
            'Right-click a layer > Create smart material':'לחיצה ימנית על שכבה > יצירת חומר חכם',
            'Place path points on the mesh':'מיקום נקודות הנתיב על הרשת',
            'Mask Editor: Edge wear':'עורך המסכות: שחיקת קצוות',
            'Height to normal strength':'עוצמת המרת גובה לנורמל',
            'UI {ms} ms':'ממשק {ms} אלפיות שנייה',
            'Offset {axis}':'היסט {axis}',
            'Source  U {u}  V {v}':'מקור  U {u}  V {v}',
        }
        for key,value in samples.items():
            self.assertEqual(locale[('',key)],value,key)
            self.assertIsNone(sync_app.text_problem('',key,value))

    def test_source_parity_and_no_common_retranslations(self):
        self.assertFalse(self.common_errors)
        self.assertFalse(self.source_errors)
        for key,value in self.app.items():
            self.assertTrue(key in self.source or key in self.additional_source_keys, key)
            self.assertNotIn(key,self.common)
            self.assertIsNone(sync_app.text_problem(*key,value))
            self.assertEqual(unicodedata.normalize('NFC',value),value)
            self.assertFalse(any(c in value for c in '\u200e\u200f\u202a\u202b\u202c\u202d\u202e\u2066\u2067\u2068\u2069'))

    def test_latest_source_backed_common_ui_labels(self):
        # These live English labels occur in the official locale source columns,
        # and were absent from both Hebrew catalogs at the previous branch HEAD.
        expected = {
            'Black': 'שחור',
            'Details': 'פרטים',
            'Dismiss': 'סגירה',
            'Distance': 'מרחק',
            'Divide': 'חלוקה',
            'Edit': 'עריכה',
            'Height': 'גובה',
            'Radius': 'רדיוס',
            'Restore': 'שחזור',
            'Settings': 'הגדרות',
            'Subtract': 'חיסור',
            'Width': 'רוחב',
        }
        locale=resolved(ROOT,'he-IL')
        for english,hebrew in expected.items():
            with self.subTest(english=english):
                key=('',english)
                self.assertIn(key,self.additional_source_keys)
                self.assertNotIn(key,self.common)
                self.assertEqual(self.app[key],hebrew)
                self.assertEqual(locale[key],hebrew)
                self.assertIsNone(sync_app.text_problem('',english,hebrew))

    def test_new_pbr_short_labels_are_source_grounded_and_resolve(self):
        # The 15 newly added channel abbreviations are separate from long-form
        # labels and must stay resolvable after future catalog growth.
        expected = {
            'aniso': 'אניזוטרופיה',
            'aniso angle': 'זווית אניזוטרופיה',
            'ao': 'חסימת אור סביבתית',
            'coat': 'ציפוי',
            'coat nrm': 'נורמל ציפוי',
            'emiss': 'פליטה',
            'gloss': 'ברק',
            'height': 'גובה',
            'metal': 'מתכתיות',
            'nrm': 'נורמל',
            'op': 'אטימות',
            'rough': 'חספוס',
            'sheen': 'ברק משי',
            'spec': 'ספקולרי',
            'transm': 'העברת אור',
        }
        locale = resolved(ROOT,'he-IL')
        for english, hebrew in expected.items():
            with self.subTest(english=english):
                key = ('', english)
                self.assertIn(key, self.additional_source_keys)
                self.assertEqual(self.app[key], hebrew)
                self.assertEqual(locale[key], hebrew)
                self.assertIsNone(sync_app.text_problem('', english, hebrew))

    def test_previous_nine_curated_values_preserved(self):
        preserved={
            'Alphas':'תבניות אלפא',
            'Brushed lines':'קווים מוברשים',
            'Flatten layer':'שיטוח שכבה',
            'Displacement (height channel)':'תזוזה (ערוץ גובה)',
            'Add High-Poly Meshes':'הוסף רשתות עתירות מצולעים',
            'Coat normal':'נורמל שכבת הציפוי',
            'Coat roughness':'חספוס שכבת הציפוי',
            'coat rough':'חספוס שכבת הציפוי',
            'Bake mesh maps':'אפיית מפות רשת'
        }
        for source,he in preserved.items():
            self.assertEqual(self.app[('',source)],he)

    def test_hebrew_inheritance_and_real_runtime_labels(self):
        locale=resolved(ROOT,'he-IL')
        self.assertEqual(locale[('','Brush')],'מברשת')
        self.assertEqual(locale[('','Add black mask')],'הוסף מסכה שחורה')
        self.assertEqual(locale[('','Export')],'ייצוא')
        self.assertEqual(locale[('','Fill')],'מילוי')
        self.assertEqual(locale[('','About TextureCraft...')],'אודות TextureCraft...')
        self.assertEqual(len(set(locale)),len(locale))

    def test_quick_wins_resolve_and_preserve_format_placeholders(self):
        locale=resolved(ROOT,'he-IL')
        samples={
            '2D view only':'תצוגת דו־ממד בלבד',
            '3D view only':'תצוגת תלת־ממד בלבד',
            'Add anchor point':'הוספת נקודת עיגון',
            'Add folder':'הוסף תיקייה',
            'Advanced settings':'הגדרות מתקדמות',
            'AO intensity':'עוצמת הסתרה סביבתית',
            'Bake Selected Textures ({n})':'אפיית הטקסטורות שנבחרו ({n})',
            'Baked maps imported: {count}':'יובאו מפות אפויות: {count}',
            '0 turns autosave off. Copies go to {folder}':'0 משבית שמירה אוטומטית. עותקים נשמרים בתיקייה {folder}',
            'Built-in: {mesh}':'מובנה: {mesh}',
        }
        for english,hebrew in samples.items():
            self.assertEqual(locale[('',english)],hebrew,english)
            self.assertIsNone(sync_app.text_problem('',english,hebrew))
        self.assertGreaterEqual(len(self.app),958)


    def test_round3_save_open_prompts_and_paths(self):
        locale=resolved(ROOT,'he-IL')
        examples={
            'Save as...':'שמירה בשם...',
            'Save a copy...':'שמירת עותק...',
            'Save the changes to {name} before closing?':'לשמור את השינויים ב־{name} לפני הסגירה?',
            "Couldn't open {name}: {error}":'לא ניתן לפתוח את {name}: {error}',
            'Saved {path}':'נשמר: {path}',
        }
        for source,hebrew in examples.items():
            self.assertEqual(locale[('',source)],hebrew,source)
            self.assertIsNone(sync_app.text_problem('',source,hebrew))

    def test_round3_import_export_results(self):
        locale=resolved(ROOT,'he-IL')
        examples={
            "Couldn't export textures: {error}":'לא ניתן לייצא טקסטורות: {error}',
            "Couldn't import {name}: {error}":'לא ניתן לייבא את {name}: {error}',
            'Exported mesh to {path}':'הרשת יוצאה אל {path}',
            'Not imported: {files}':'לא יובאו: {files}',
            'Import resources...':'ייבוא משאבים...',
        }
        for source,hebrew in examples.items():
            self.assertEqual(locale[('',source)],hebrew,source)
            self.assertIsNone(sync_app.text_problem('',source,hebrew))

    def test_round3_layer_and_mask_actions(self):
        locale=resolved(ROOT,'he-IL')
        examples={
            'Layer mask':'מסכת שכבה',
            'Paint layer':'שכבת צביעה',
            'Remove layer(s)':'הסר שכבות',
            'Invert mask':'הפוך מסכה',
            'Select a layer':'בחר שכבה',
        }
        for source,hebrew in examples.items():
            self.assertEqual(locale[('',source)],hebrew,source)

    def test_round3_rendering_progress_and_errors(self):
        locale=resolved(ROOT,'he-IL')
        examples={
            'GPU error: {error}':'שגיאת GPU: {error}',
            'No GPU renderer available':'אין מנוע רינדור זמין ב־GPU',
            'Rendering':'רינדור',
            'Rendering on CPU: {n} / {max} samples, {s} s':'רינדור במעבד: {n} מתוך {max} דגימות, {s} שנ׳',
            "Couldn't save the render: {error}":'לא ניתן לשמור את תוצאת הרינדור: {error}',
        }
        for source,hebrew in examples.items():
            self.assertEqual(locale[('',source)],hebrew,source)
            self.assertIsNone(sync_app.text_problem('',source,hebrew))

    def test_round3_settings_reset_and_resolution(self):
        locale=resolved(ROOT,'he-IL')
        examples={
            'Reset all':'איפוס הכול',
            'Reset to defaults':'איפוס לברירות המחדל',
            'Reset UI':'איפוס הממשק',
            'Texture resolution':'רזולוציית טקסטורה',
            'Save UI Layout':'שמירת פריסת הממשק',
        }
        for source,hebrew in examples.items():
            self.assertEqual(locale[('',source)],hebrew,source)


    def _assert_round4_cases(self, examples):
        locale=resolved(ROOT,'he-IL')
        for english,hebrew in examples.items():
            with self.subTest(english=english):
                self.assertIn(('',english),self.source)
                self.assertNotIn(('',english),self.common)
                self.assertEqual(self.app[('',english)],hebrew)
                self.assertEqual(locale[('',english)],hebrew)
                self.assertIsNone(sync_app.text_problem('',english,hebrew))

    def test_round4_viewport_navigation_labels(self):
        self._assert_round4_cases({
            'Camera settings':'הגדרות מצלמה',
            'Default camera':'מצלמת ברירת המחדל',
            'Field of view (deg)':'שדה ראייה (מעלות)',
            'Hide UI':'הסתר את הממשק',
            'Show UV wireframe in the 2D view':'הצג קווי רשת UV בתצוגת דו־ממד',
            'Show wireframe in the 3D view':'הצג קווי רשת בתצוגת תלת־ממד',
            'Screenshot of the viewport':'צילום אזור התצוגה',
            'Viewport':'אזור תצוגה',
        })

    def test_round4_paint_color_and_materials(self):
        self._assert_round4_cases({
            'Base color':'צבע בסיס',
            'Blur Directional':'טשטוש כיווני',
            'Color Balance':'איזון צבעים',
            'Color picker':'בוחר צבעים',
            'Flow and opacity':'זרימה ואטימות',
            'Polygon fill':'מילוי מצולע',
            'Smart materials':'חומרים חכמים',
            'Stroke opacity':'אטימות משיכת המכחול',
        })

    def test_round4_uv_and_mesh_workflows(self):
        self._assert_round4_cases({
            'High Definition Mesh (HP)':'רשת ברזולוציה גבוהה (HP)',
            'Mesh map: {map}':'מפת רשת: {map}',
            'Normal map format':'פורמט מפת נורמל',
            'Spherical projection':'הקרנה כדורית',
            'Tri-planar projection':'הקרנה תלת־מישורית',
            'UV tiles (UDIM)':'אריחי UV (UDIM)',
            'UV checker':'תבנית בדיקת UV',
            'UV island fill':'מילוי אי UV',
        })

    def test_round4_assets_and_templates(self):
        self._assert_round4_cases({
            'Assets':'נכסים',
            'Deleted template "{name}"':'התבנית "{name}" נמחקה',
            'Imported template "{name}"':'התבנית "{name}" יובאה',
            'My smart materials':'החומרים החכמים שלי',
            'Remove unused resources...':'הסרת משאבים שאינם בשימוש...',
            'Rescan the library folder':'סריקה מחדש של תיקיית הספרייה',
            'Search all assets':'חיפוש בכל הנכסים',
            'Smart material saved to {path}':'החומר החכם נשמר ב־{path}',
        })

    def test_round4_settings_shortcuts_and_status(self):
        self._assert_round4_cases({
            'Details saved to {file}':'הפרטים נשמרו ב־{file}',
            'Edit keyboard shortcuts...':'עריכת קיצורי מקלדת...',
            'Import MaterialX material...':'ייבוא חומר MaterialX...',
            'Not saved: {error}':'לא נשמר: {error}',
            'Reset to {shortcut}':'איפוס ל־{shortcut}',
            'Save the changes to {name} first?':'האם לשמור תחילה את השינויים ב־{name}?',
            'Texture Set Settings':'הגדרות ערכת טקסטורות',
            '{n} items share a shortcut; the first one listed wins.':'ל־{n} פריטים אותו קיצור; הפריט הראשון ברשימה יקבל עדיפות.',
        })


    def test_round5_layer_editing(self):
        self._assert_round4_cases({
            "1 effect": "אפקט אחד",
            "Add an anchor point to a layer below to use its stack as a mask.": "הוסף נקודת עיגון לשכבה שמתחת כדי להשתמש במערך השכבות שלה כמסכה.",
            "Anchor: {name}": "עיגון: {name}",
            "Anchor: {name} ({channel})": "עיגון: {name} ({channel})",
            "Double-click or press Enter to edit": "לחץ פעמיים או הקש Enter לעריכה",
            "Editing quick mask: paint where painting is allowed (U to finish)": "עריכת מסכה מהירה: צבע היכן שהצביעה מותרת (U לסיום)",
            "Folders": "תיקיות",
            "Invert mask value": "הפוך את ערך המסכה",
            "Mask effects": "אפקטים של המסכה",
            "Paint on the mesh to edit this mask. Use the Material tab's mask value for gray.": "צבע על הרשת כדי לערוך את המסכה. השתמש בערך המסכה בלשונית החומר עבור אפור.",
            "Layer memory: {mb} MB": "זיכרון שכבה: {mb} MB",
        })

    def test_round5_painting_effects(self):
        self._assert_round4_cases({
            "Accent color": "צבע הדגשה",
            "Brightness": "בהירות",
            "Curvature": "עקמומיות",
            "Difference": "הפרש",
            "Emissive": "פליטת אור",
            "Hardness": "קשיחות",
            "Levels": "רמות",
            "Position gradient": "מעבר צבע לפי מיקום",
            "Spray": "ריסוס",
            "Size jitter": "שונות בגודל",
        })

    def test_round5_mesh_baking(self):
        self._assert_round4_cases({
            "Bake an ID map (F8) to pick colors from the mesh.": "אפה מפת מזהים (F8) כדי לדגום צבעים מהרשת.",
            "Baking {map} ({n}/{total}), texture set {set} of {sets}": "אופה את {map} ({n}/{total}), ערכת טקסטורות {set} מתוך {sets}",
            "Cage Type": "סוג כלוב",
            "Dilation Width": "רוחב הרחבה",
            "Input high": "גבול קלט עליון",
            "Mesh Normals Based": "מבוסס על נורמלים של הרשת",
            "Normal Invert": "היפוך נורמל",
            "Output Size": "גודל פלט",
            "Samples": "דגימות",
            "World Space Normal": "נורמל במרחב העולם",
            "Baking {map} ({n}/{total})": "אופה את {map} ({n}/{total})",
            "Cage: {name}": "כלוב: {name}",
            "High poly: {name}": "רשת עתירת מצולעים: {name}",
            "UV tile {tile} resolution": "רזולוציית אריח UV {tile}",
        })

    def test_round5_file_and_environment(self):
        self._assert_round4_cases({
            "Check All": "סמן הכול",
            "Copy RGB values": "העתק ערכי RGB",
            "Download as": "הורדה בתור",
            "Environments": "סביבות",
            "Keyboard": "מקלדת",
            "no GPU": "אין GPU",
            "Press a key...": "הקש על מקש...",
            "Search commands or keys": "חיפוש פקודות או קיצורי מקלדת",
            "TextureCraft project": "פרויקט TextureCraft",
            "Version: {version} ({gpu})": "גרסה: {version} ({gpu})",
            "Downloaded {name}": "הורד {name}",
            "Environment: {name}": "סביבה: {name}",
            "Loaded UI layout {path}": "פריסת הממשק נטענה מ־{path}",
            "UI layout: {error}": "פריסת הממשק: {error}",
        })

    def test_round5_render_status_and_errors(self):
        self._assert_round4_cases({
            "Done on CPU: {n} / {max} samples, {s} s": "הושלם במעבד: {n} מתוך {max} דגימות, {s} שניות",
            "Follow Newest Map": "עקוב אחר המפה החדשה ביותר",
            "Lens size: how quickly things blur away from the focus distance": "גודל העדשה: קצב הטשטוש עם ההתרחקות ממרחק המיקוד",
            "Light / direction": "אור / כיוון",
            "Overlay": "שכבת־על",
            "Path trace on the processor instead of the graphics card (slower)": "עקיבת נתיבים במעבד במקום בכרטיס הגרפי (איטית יותר)",
            "Rendering (path traced)": "רינדור (עקיבת נתיבים)",
            "Show or hide the texture set": "הצג או הסתר את ערכת הטקסטורות",
            "The file dialog couldn't be shown: {error}": "לא ניתן להציג את חלונית הקבצים: {error}",
            "Texture sets": "ערכות טקסטורות",
            "Done on GPU: {n} / {max} samples, {s} s": "הושלם במעבד הגרפי: {n} מתוך {max} דגימות, {s} שניות",
        })

    def test_round9_source_locked_hebrew_additions(self):
        locale=resolved(ROOT,'he-IL')
        examples={
            "min": "דק׳",
            "Free and open source (MIT OR Apache-2.0), written in Rust.": "קוד פתוח וחופשי (MIT OR Apache-2.0), נכתב בשפת Rust.",
            "Image: {name}": "תמונה: {name}",
            "Not enough GPU memory to show texture set \"{name}\" fully: lower the viewport Texture resolution in Settings. Painting and export are not affected.": "אין מספיק זיכרון GPU להצגת ערכת הטקסטורות \"{name}\" במלואה: הקטן את רזולוציית הטקסטורה בחלון התצוגה דרך ההגדרות. הצביעה והייצוא אינם מושפעים.",
            "Works in texture space, like the blur: it does not cross UV seams.": "פועל במרחב הטקסטורה, כמו טשטוש: אינו חוצה תפרי UV.",
        }
        for english,hebrew in examples.items():
            self.assertIn(('',english),self.source)
            self.assertNotIn(('',english),self.common)
            self.assertEqual(self.app[('',english)],hebrew)
            self.assertEqual(locale[('',english)],hebrew)
            self.assertIsNone(sync_app.text_problem('',english,hebrew))

    def test_round10_remaining_visible_labels_resolve_in_hebrew(self):
        locale=resolved(ROOT,'he-IL')
        examples={
            "BBox":"תיבת תיחום",
            "Go to camera 9":"מעבר למצלמה 9",
            "ID":"מזהה",
            "MatFinish Hammered":"גימור חומר: רקוע",
            "Pin light":"הצמדת מקור אור",
            "Tangent | Wrap":"משיק | עטיפה",
        }
        for english,hebrew in examples.items():
            self.assertIn(('',english),self.source)
            self.assertNotIn(('',english),self.common)
            self.assertEqual(self.app[('',english)],hebrew)
            self.assertEqual(locale[('',english)],hebrew)
            self.assertIsNone(sync_app.text_problem('',english,hebrew))


    def test_round11_new_visible_blend_and_controls(self):
        locale=resolved(ROOT,"he-IL")
        examples={
            'Exclusion': 'החרגה',
            'Full': 'מלא',
            'High': 'גבוה',
            'Level': 'רמה',
            'Low': 'נמוך',
            'Luminosity': 'בהיקות',
            'Method': 'שיטה',
            'Midtones': 'גווני ביניים',
            'Mode': 'מצב',
            'Multiply': 'הכפלה',
            'Never': 'לעולם לא',
            'Offset': 'היסט',
            'Random': 'אקראי',
            'Screen': 'מסך',
            'Shadows': 'צללים',
            'Uniform': 'אחיד',
            'Violet': 'סגול',
        }
        for english,hebrew in examples.items():
            self.assertIn(("",english),self.source)
            self.assertNotIn(("",english),self.common)
            self.assertEqual(self.app[("",english)],hebrew)
            self.assertEqual(locale[("",english)],hebrew)
            self.assertIsNone(sync_app.text_problem("",english,hebrew))

if __name__=='__main__':
    unittest.main()
