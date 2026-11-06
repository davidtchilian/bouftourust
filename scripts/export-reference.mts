// Exécution ponctuelle : node ../bouftourush/node_modules/tsx/dist/cli.mjs scripts/export-reference.mts
import {writeFileSync} from 'node:fs';
import {balance,spells,classBalance} from '../../bouftourush/src/data/balance.ts';
import {classes} from '../../bouftourush/src/data/classes.ts';
import {dungeons,dungeonModes} from '../../bouftourush/src/data/dungeons.ts';
import {monsters} from '../../bouftourush/src/data/monsters.ts';
import {items,sets,slots,slotNames} from '../../bouftourush/src/data/items.ts';
import {spellDescriptions} from '../../bouftourush/src/data/spellDescriptions.ts';
import {resalePrices} from '../../bouftourush/src/data/resale.ts';
writeFileSync('assets/data/game.json',JSON.stringify({balance,spells,classBalance,classes,dungeons,dungeonModes,monsters,items,sets,slots,slotNames,resalePrices,spellDescriptions},null,2)+'\n');
