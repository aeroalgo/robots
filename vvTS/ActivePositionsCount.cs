using System;
using System.Collections.Generic;
using System.Linq;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000D9 RID: 217
	[HandlerCategory("vvTrade"), HandlerName("Количество активных позиций")]
	public class ActivePositionsCount : IOneSourceHandler, IDoubleReturns, IValuesHandler, IHandler, ISecurityInputs
	{
		// Token: 0x06000712 RID: 1810 RVA: 0x0001F9B8 File Offset: 0x0001DBB8
		public double Execute(ISecurity sec, int barNum)
		{
			IEnumerable<IPosition> activeForBar = sec.get_Positions().GetActiveForBar(barNum);
			if (activeForBar.Count<IPosition>() < 1)
			{
				return 0.0;
			}
			IEnumerable<IPosition> closedOrActiveForBar = sec.get_Positions().GetClosedOrActiveForBar(barNum);
			int num = 0;
			foreach (IPosition current in closedOrActiveForBar)
			{
				if (current.IsActiveForbar(barNum))
				{
					num++;
				}
			}
			return (double)num;
		}
	}
}
