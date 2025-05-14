using System;
using System.Collections.Generic;
using System.Linq;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000E0 RID: 224
	[HandlerCategory("vvTrade"), HandlerName("Последние - 3 шорта подряд в убыток")]
	public class Last3LossedShorts : IBar2BoolHandler, IOneSourceHandler, IBooleanReturns, IValuesHandler, IHandler, ISecurityInputs
	{
		// Token: 0x06000720 RID: 1824 RVA: 0x0001FD3C File Offset: 0x0001DF3C
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
				if (num < 1)
				{
					return false;
				}
			}
			return !closedOrActiveForBar.ElementAt(num - 1).IsActiveForbar(barNum) && !closedOrActiveForBar.ElementAt(num - 2).IsActiveForbar(barNum) && (closedOrActiveForBar.ElementAt(num).get_IsShort() && closedOrActiveForBar.ElementAt(num - 1).get_IsShort() && closedOrActiveForBar.ElementAt(num - 2).get_IsShort() && closedOrActiveForBar.ElementAt(num).Profit() < 0.0 && closedOrActiveForBar.ElementAt(num - 1).Profit() < 0.0) && closedOrActiveForBar.ElementAt(num - 2).Profit() < 0.0;
		}
	}
}
