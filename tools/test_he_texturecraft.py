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

    def test_hebrew_app_catalog_has_527_distinct_source_rows(self):
        self.assertFalse(self.parse_errors)
        self.assertEqual(len(self.app),527)
        self.assertTrue(all(c=='' for c,s in self.app))
        self.assertTrue(all(s and v for (c,s),v in self.app.items()))

    def test_source_parity_and_no_common_retranslations(self):
        self.assertFalse(self.common_errors)
        self.assertFalse(self.source_errors)
        for key,value in self.app.items():
            self.assertIn(key,self.source)
            self.assertNotIn(key,self.common)
            self.assertIsNone(sync_app.text_problem(*key,value))
            self.assertEqual(unicodedata.normalize('NFC',value),value)
            self.assertFalse(any(c in value for c in '\u200e\u200f\u202a\u202b\u202c\u202d\u202e\u2066\u2067\u2068\u2069'))

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
        self.assertEqual(len(self.app),527)


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

if __name__=='__main__':
    unittest.main()
