using System;
using System.Collections.Generic;
using System.Linq;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000DD RID: 221
	[HandlerCategory("vvTrade"), HandlerName("Последние 2 закрытых\nсделки подряд - Short")]
	public class Last2TradesWereShort : IBar2BoolHandler, IOneSourceHandler, IBooleanReturns, IValuesHandler, IHandler, ISecurityInputs
	{
		// Token: 0x0600071A RID: 1818 RVA: 0x0001FB80 File Offset: 0x0001DD80
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
			return !closedOrActiveForBar.ElementAt(num - 1).IsActiveForbar(barNum) && closedOrActiveForBar.ElementAt(num).get_IsShort() && closedOrActiveForBar.ElementAt(num - 1).get_IsShort();
		}
	}
}
