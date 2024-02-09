using System;
using System.Collections.Generic;
using System.Linq;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000E2 RID: 226
	[HandlerCategory("vvTrade"), HandlerName("Последние - 3 шорта подряд в прибыль")]
	public class Last3ProfitShorts : IBar2BoolHandler, IOneSourceHandler, IBooleanReturns, IValuesHandler, IHandler, ISecurityInputs
	{
		// Token: 0x06000724 RID: 1828 RVA: 0x0001FEE0 File Offset: 0x0001E0E0
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
			return !closedOrActiveForBar.ElementAt(num - 1).IsActiveForbar(barNum) && !closedOrActiveForBar.ElementAt(num - 2).IsActiveForbar(barNum) && (closedOrActiveForBar.ElementAt(num).get_IsShort() && closedOrActiveForBar.ElementAt(num - 1).get_IsShort() && closedOrActiveForBar.ElementAt(num - 2).get_IsShort() && closedOrActiveForBar.ElementAt(num).Profit() > 0.0 && closedOrActiveForBar.ElementAt(num - 1).Profit() > 0.0) && closedOrActiveForBar.ElementAt(num - 2).Profit() > 0.0;
		}
	}
}
