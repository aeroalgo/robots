using System;
using System.Collections.Generic;
using System.Linq;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000DC RID: 220
	[HandlerCategory("vvTrade"), HandlerName("Количество активных позиций Short")]
	public class ActiveShortsCount : IOneSourceHandler, IDoubleReturns, IValuesHandler, IHandler, ISecurityInputs
	{
		// Token: 0x06000718 RID: 1816 RVA: 0x0001FAF8 File Offset: 0x0001DCF8
		public double Execute(ISecurity sec, int barNum)
		{
			IEnumerable<IPosition> activeForBar = sec.get_Positions().GetActiveForBar(barNum);
			if (activeForBar.Count<IPosition>() < 1)
			{
				return 0.0;
			}
			double num = 0.0;
			foreach (IPosition current in activeForBar)
			{
				if (current.get_IsShort())
				{
					num += 1.0;
				}
			}
			return num;
		}
	}
}
