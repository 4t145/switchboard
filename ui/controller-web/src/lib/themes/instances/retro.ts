import type { Theme } from "..";

const RED: string = '#B3514B';
const LIGHT_YELLOW: string = '#DFD6CB';
const DARK_RED: string = '#8B3A37';

const spacecraftButtonStyles = `
  px-6 py-3 
  font-bold text-lg
  border-4 border-[${RED}]
  bg-[${RED}]
  text-white
  cursor-pointer
  transition-all duration-75
  
  /* 3D 立体效果 */
  box-shadow: 
    0 8px 0 ${DARK_RED},
    0 8px 20px rgba(0, 0, 0, 0.3),
    inset 0 -2px 0 rgba(0, 0, 0, 0.2),
    inset 0 2px 0 rgba(255, 255, 255, 0.3)
  
  /* 发光效果 */
  text-shadow: 0 0 10px rgba(255, 200, 0, 0.8)
  
  /* 按下效果 */
  active:translate-y-2
  active:box-shadow: 
    0 4px 0 ${DARK_RED},
    0 4px 10px rgba(0, 0, 0, 0.2),
    inset 0 -2px 0 rgba(0, 0, 0, 0.2),
    inset 0 2px 0 rgba(255, 255, 255, 0.3)
  
  /* 悬停发光 */
  hover:shadow-[
    0_8px_0_${DARK_RED},
    0_8px_20px_rgba(0,0,0,0.3),
    inset_0_-2px_0_rgba(0,0,0,0.2),
    inset_0_2px_0_rgba(255,255,255,0.3),
    0_0_20px_rgba(255,200,0,0.6)
  ]
`;

const retroTheme: Theme = {
    name: 'retro',
    styles: {
        'Button': spacecraftButtonStyles,
        'Card': `bg-[${LIGHT_YELLOW}] border-4 border-[${RED}] p-4 rounded-lg shadow-lg`,
        'Header': 'bg-green-500 text-white text-3xl font-extrabold p-6',
        'Footer': 'bg-blue-500 text-white text-center p-4 mt-8',
    }
}

export default retroTheme;