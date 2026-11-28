"""Source-locked Hebrew PhotoCraft interface labels introduced November 2026."""
import sys
import unittest
import unicodedata
from pathlib import Path

ROOT=Path(__file__).resolve().parent.parent
sys.path.insert(0,str(ROOT/'tools'))
import sync_app

EXPECTED={'Align bottom edges': 'יישר את הקצוות התחתונים',
 'Align top edges': 'יישר את הקצוות העליונים',
 'Align vertical centers': 'יישר את המרכזים האנכיים',
 'Auto-nest Layers': 'קינון שכבות אוטומטי',
 'Auto-size Canvas': 'התאם את גודל משטח העבודה אוטומטית',
 'Clear Recent File List': 'נקה את רשימת הקבצים האחרונים',
 'Clip Thumbnails to Document Bounds': 'חתוך תמונות ממוזערות לגבולות המסמך',
 'Consolidate All to Here': 'אחד את כל החלונות לכאן',
 'Contextual Task Bar': 'סרגל משימות הקשרי',
 'Could not transform because the layer is locked.': 'לא ניתן לשנות את צורת השכבה משום שהיא נעולה.',
 'Default Foreground and Background Colors (D)': 'צבעי ברירת המחדל של החזית והרקע (D)',
 'Distribute horizontally': 'פזר באופן אופקי',
 'Distribute vertically': 'פזר באופן אנכי',
 'Edit Brush…': 'ערוך מברשת…',
 'Embed All Linked': 'הטמע את כל הפריטים המקושרים',
 'Embed Linked': 'הטמע פריט מקושר',
 'Expand layer group on click': 'הרחב קבוצת שכבות בלחיצה',
 'Hide bar': 'הסתר סרגל',
 'Keep Relative Position During Layer Reordering': 'שמור על המיקום היחסי בעת שינוי סדר השכבות',
 'Large Thumbnails': 'תמונות ממוזערות גדולות',
 'Medium Thumbnails': 'תמונות ממוזערות בינוניות',
 'More options': 'אפשרויות נוספות',
 'Move bar': 'הזז סרגל',
 'Move to New Window': 'העבר לחלון חדש',
 'New Document…': 'מסמך חדש…',
 'No Thumbnails': 'ללא תמונות ממוזערות',
 'Not enough grayscale documents for this mode.': 'אין מספיק מסמכים בגווני אפור למצב זה.',
 'Open Document…': 'פתח מסמך…',
 'Open a document to select a color range.': 'פתח מסמך כדי לבחור טווח צבעים.',
 'Pin bar position': 'קבע את מיקום הסרגל',
 'Plugins': 'תוספים',
 'Plugins…': 'תוספים…',
 'Reset bar position': 'אפס את מיקום הסרגל',
 'Reveal in Explorer': 'הצג בסייר הקבצים',
 'Save canceled': 'השמירה בוטלה',
 'Scroll All Windows': 'גלול את כל החלונות',
 'Show hover bounds from Layers': 'הצג גבולות בריחוף בחלונית השכבות',
 'Show layer bounds on hover': 'הצג גבולות שכבה בריחוף',
 'Skip transform when placing': 'דלג על שינוי צורה בעת הצבה',
 'Small Thumbnails': 'תמונות ממוזערות קטנות',
 'Switch Foreground and Background Colors (X)': 'החלף בין צבע החזית לצבע הרקע (X)'}

class PhotoCraftSourceLabelsTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.he,errors=sync_app.parse((ROOT/'locales/he/photocraft.tsv').read_text('utf-8'))
        assert not errors,errors
        cls.common,errors=sync_app.parse((ROOT/'locales/he/common.tsv').read_text('utf-8'))
        assert not errors,errors
        cls.sources=[]
        for lang in ('ar','de','es','fr','ja','ko'):
            source,errors=sync_app.parse((ROOT/f'locales/{lang}/photocraft.tsv').read_text('utf-8'))
            assert not errors,(lang,errors)
            cls.sources.append(source)

    def test_original_hebrew_labels_resolve_without_colliding_with_common(self):
        for english,hebrew in EXPECTED.items():
            with self.subTest(english=english):
                self.assertEqual(self.he[('',english)],hebrew)
                self.assertNotIn(('',english),self.common)

    def test_english_keys_exist_in_multiple_original_source_catalogs(self):
        for english,hebrew in EXPECTED.items():
            with self.subTest(english=english):
                key=('',english)
                self.assertGreaterEqual(sum(key in source for source in self.sources),3)
                self.assertIsNone(sync_app.text_problem('',english,hebrew))
                self.assertEqual(unicodedata.normalize('NFC',hebrew),hebrew)

if __name__=='__main__':
    unittest.main()
