import en from './en';
import pt from './pt';
import es from './es';
import fr from './fr';
import it from './it';
import de from './de';

const resources = { en, pt, es, fr, it, de };

let current = 'en';   // default

export default {
  setLanguage: (lang) => { current = lang in resources ? lang : 'en'; },
  t: (key) => resources[current][key] ?? key,
  current: () => current,
};
