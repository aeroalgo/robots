using System;
using System.Collections.Generic;
using System.Linq;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000DF RID: 223
	[HandlerCategory("vvTrade"), HandlerName("Последние - 2 шорта подряд в убыток")]
	public class Last2LossedShorts : IBar2BoolHandler, IOneSourceHandler, IBooleanReturns, IValuesHandler, IHandler, ISecurityInputs
	{
		// Token: 0x0600071E RID: 1822 RVA: 0x0001FC88 File Offset: 0x0001DE88
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
			return !closedOrActiveForBar.ElementAt(num - 1).IsActiveForbar(barNum) && (closedOrActiveForBar.ElementAt(num).get_IsShort() && closedOrActiveForBar.ElementAt(num - 1).get_IsShort() && closedOrActiveForBar.ElementAt(num).Profit() < 0.0) && closedOrActiveForBar.ElementAt(num - 1).Profit() < 0.0;
		}
	}
}
