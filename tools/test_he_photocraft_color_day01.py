"""Novel source-keyed Hebrew PhotoCraft color/document UI strings, day shift 2026-11-19."""
import sys
import unittest
import unicodedata
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent
sys.path.insert(0,str(ROOT/'tools'))
import sync_app
EXPECTED={'A picked document is no longer a grayscale document of the same size.': 'המסמך שנבחר כבר אינו מסמך בגווני אפור באותם ממדים.', 'A picked document is no longer open.': 'המסמך שנבחר כבר אינו פתוח.', 'Canceled opening {name}': 'פתיחת {name} בוטלה', 'Center the gradient (offset 0, 0)': 'מרכז את מעבר הצבע (היסט 0, 0)', 'Click to fill similar colors': 'לחץ למילוי אזורים בצבעים דומים', 'Click to restore the current color': 'לחץ לשחזור הצבע הנוכחי', 'Click to sample the foreground color  ·  {key}-click for background': 'לחץ לדגימת צבע החזית  ·  לחיצה עם {key} לדגימת צבע הרקע', 'Color Blindness - Deuteranopia-type': 'עיוורון צבעים מסוג דויטרנופיה', 'Color Blindness - Protanopia-type': 'עיוורון צבעים מסוג פרוטנופיה', 'Drag around the center to rotate the view  ·  Shift constrains to 15°': 'גרור סביב המרכז כדי לסובב את התצוגה  ·  Shift מגביל לקפיצות של 15°', 'Internet Standard RGB (sRGB)': 'RGB תקני לאינטרנט (sRGB)', 'Legacy Macintosh RGB (Gamma 1.8)': 'RGB ישן של Macintosh (גאמה 1.8)', 'Make Shape': 'צור צורה', 'Message': 'הודעה', 'Not a web color: click for the nearest web color': 'זה אינו צבע אינטרנט: לחץ לבחירת צבע האינטרנט הקרוב ביותר', 'Out of gamut for printing: click for the nearest printable color': 'מחוץ לטווח הצבעים להדפסה: לחץ לבחירת הצבע הקרוב ביותר שניתן להדפיס', 'Pick a different document for each channel.': 'בחר מסמך אחר לכל ערוץ.', 'Sample: foreground color': 'דגימה: צבע החזית', 'Scratch Size Limit (MB, 0 = No Limit)': 'מגבלת נפח לדיסק עבודה זמני (MB, 0 = ללא הגבלה)', 'Shrinkwrap Canvas on Save': 'התאם את גבולות משטח העבודה לתוכן בעת שמירה', 'Memory usage percent': 'אחוז השימוש בזיכרון', 'Rotate View Left': 'סובב את התצוגה שמאלה', 'Rotate View Right': 'סובב את התצוגה ימינה', 'Save Full-Size Composite in .pcraft Files': 'שמור תמונה מורכבת בגודל מלא בקובצי .pcraft', "Save the settings to each raw's sidecar without opening it": 'שמור את ההגדרות בקובץ נלווה לכל קובץ RAW בלי לפתוח אותו', 'Some settings could not be saved; the raws stay open': 'לא ניתן לשמור חלק מההגדרות; קובצי RAW נשארים פתוחים', 'Nothing to save: these raws are at their default settings': 'אין מה לשמור: קובצי RAW אלה משתמשים בהגדרות ברירת המחדל', "No settings saved: these raws can't keep a sidecar here": 'לא נשמרו הגדרות: אי אפשר לשמור כאן קובץ נלווה לקובצי RAW אלה'}
LANGS=('ar','de','es','fr','ja','ko')

class PhotoCraftColorDay01Tests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.app,errors=sync_app.parse((ROOT/'locales/he/photocraft.tsv').read_text('utf-8'))
        assert not errors, errors
        cls.common,errors=sync_app.parse((ROOT/'locales/he/common.tsv').read_text('utf-8'))
        assert not errors, errors
        cls.sources=[]
        for lang in LANGS:
            src,errors=sync_app.parse((ROOT/f'locales/{lang}/photocraft.tsv').read_text('utf-8'))
            assert not errors,(lang,errors)
            cls.sources.append(src)

    def test_exact_hebrew_and_noncollision(self):
        for english,hebrew in EXPECTED.items():
            with self.subTest(english=english):
                self.assertEqual(self.app[('',english)],hebrew)
                self.assertNotIn(('',english),self.common)

    def test_source_provenance_unicode_and_placeholders(self):
        for english,hebrew in EXPECTED.items():
            with self.subTest(english=english):
                key=('',english)
                self.assertGreaterEqual(sum(key in source for source in self.sources),3)
                self.assertEqual(unicodedata.normalize('NFC',hebrew),hebrew)
                self.assertIsNone(sync_app.text_problem('',english,hebrew))

if __name__=='__main__':
    unittest.main()
