using System;
using System.Collections.Generic;
using System.Linq;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000DE RID: 222
	[HandlerCategory("vvTrade"), HandlerName("Последние 2 закрытых\nсделки подряд - Long")]
	public class Last2TradesWereLong : IBar2BoolHandler, IOneSourceHandler, IBooleanReturns, IValuesHandler, IHandler, ISecurityInputs
	{
		// Token: 0x0600071C RID: 1820 RVA: 0x0001FC04 File Offset: 0x0001DE04
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
			return !closedOrActiveForBar.ElementAt(num - 1).IsActiveForbar(barNum) && closedOrActiveForBar.ElementAt(num).get_IsLong() && closedOrActiveForBar.ElementAt(num - 1).get_IsLong();
		}
	}
}
