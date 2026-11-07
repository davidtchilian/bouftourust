// Export historique TypeScript : pour le nouvel équilibrage, utiliser
// python3 scripts/export-balanced-fixtures.py (données Rust courantes).
import {writeFileSync} from 'node:fs';
import {items,sets} from '../../bouftourush/src/data/items.ts';
import {newProfile,finalStats,movementSpeedBonus,awardXp,equip} from '../../bouftourush/src/game/progression/profile.ts';
import {sellDuplicates} from '../../bouftourush/src/game/progression/resale.ts';
import {damage} from '../../bouftourush/src/game/combat/damage.ts';
const cases=[];
for(const setId of Object.keys(sets))for(const level of [1,25,100]){
 const p=newProfile();p.level=level;let i=0;
 for(const item of Object.values(items).filter(i=>i.set===setId)){const uid=`fixture-${++i}`;p.inventory.push({uid,item:item.id});equip(p,uid);p.inventory.push({uid:`spare-${i}`,item:item.id});p.inventory.push({uid:`extra-${i}`,item:item.id});}
 const stats=finalStats(p),speed=movementSpeedBonus(p),sale=sellDuplicates(p);
 cases.push({setId,profile:p,stats,speed,sale});
}
const damages=[];for(const power of [0,4,100,301,10000])for(const c of [1.2,1.7,2.3,5,9])for(const buff of [1,1.6])damages.push({power,c,buff,value:damage(16,c,power,buff)});
const xp=[];for(const amount of [0,25,200,2000,1000000]){const p=newProfile();const hp=awardXp(p,amount);xp.push({amount,level:p.level,xp:p.xp,hp});}
writeFileSync('tests/reference_progression.json',JSON.stringify({cases,damages,xp},null,2)+'\n');
