using System;
using System.Collections.Generic;
using System.Linq;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000E8 RID: 232
	[HandlerCategory("vvTrade"), HandlerName("Убыток ли в 2-х посл. сделках?")]
	public class LossIn2LastTrades : IBar2BoolHandler, IOneSourceHandler, IBooleanReturns, IValuesHandler, IHandler, ISecurityInputs
	{
		// Token: 0x06000730 RID: 1840 RVA: 0x00020350 File Offset: 0x0001E550
		public bool Execute(ISecurity sec, int barNum)
		{
			IEnumerable<IPosition> closedOrActiveForBar = sec.get_Positions().GetClosedOrActiveForBar(barNum);
			if (closedOrActiveForBar.Count<IPosition>() < 2)
			{
				return false;
			}
			int num = closedOrActiveForBar.Count<IPosition>() - 1;
			while (closedOrActiveForBar.ElementAt(num).IsActiveForbar(barNum))
			{
				num--;
				if (num < 1)
				{
					return false;
				}
			}
			return !closedOrActiveForBar.ElementAt(num - 1).IsActiveForbar(barNum) && closedOrActiveForBar.ElementAt(num).Profit() < 0.0 && closedOrActiveForBar.ElementAt(num - 1).Profit() < 0.0;
		}
	}
}
