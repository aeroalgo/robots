using System;
using System.Collections.Generic;
using System.Linq;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000E4 RID: 228
	[HandlerCategory("vvTrade"), HandlerName("Последние - 3 лонга подряд в убыток")]
	public class Last3LossedLongs : IBar2BoolHandler, IOneSourceHandler, IBooleanReturns, IValuesHandler, IHandler, ISecurityInputs
	{
		// Token: 0x06000728 RID: 1832 RVA: 0x00020084 File Offset: 0x0001E284
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
			return !closedOrActiveForBar.ElementAt(num - 1).IsActiveForbar(barNum) && !closedOrActiveForBar.ElementAt(num - 2).IsActiveForbar(barNum) && (closedOrActiveForBar.ElementAt(num).get_IsLong() && closedOrActiveForBar.ElementAt(num - 1).get_IsLong() && closedOrActiveForBar.ElementAt(num - 2).get_IsLong() && closedOrActiveForBar.ElementAt(num).Profit() < 0.0 && closedOrActiveForBar.ElementAt(num - 1).Profit() < 0.0) && closedOrActiveForBar.ElementAt(num - 2).Profit() < 0.0;
		}
	}
}
