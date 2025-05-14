using System;
using System.Collections.Generic;
using System.Linq;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000E9 RID: 233
	[HandlerCategory("vvTrade"), HandlerName("Убыток ли в 3-х посл. сделках?")]
	public class LossIn3LastTrades : IBar2BoolHandler, IOneSourceHandler, IBooleanReturns, IValuesHandler, IHandler, ISecurityInputs
	{
		// Token: 0x06000732 RID: 1842 RVA: 0x000203E8 File Offset: 0x0001E5E8
		public bool Execute(ISecurity sec, int barNum)
		{
			IEnumerable<IPosition> closedOrActiveForBar = sec.get_Positions().GetClosedOrActiveForBar(barNum);
			if (closedOrActiveForBar.Count<IPosition>() < 3)
			{
				return false;
			}
			int num = closedOrActiveForBar.Count<IPosition>() - 1;
			while (closedOrActiveForBar.ElementAt(num).IsActiveForbar(barNum))
			{
				num--;
				if (num < 2)
				{
					return false;
				}
			}
			return !closedOrActiveForBar.ElementAt(num - 1).IsActiveForbar(barNum) && !closedOrActiveForBar.ElementAt(num - 2).IsActiveForbar(barNum) && (closedOrActiveForBar.ElementAt(num).Profit() < 0.0 && closedOrActiveForBar.ElementAt(num - 1).Profit() < 0.0) && closedOrActiveForBar.ElementAt(num - 2).Profit() < 0.0;
		}
	}
}
