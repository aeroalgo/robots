using System;
using System.Collections.Generic;
using System.Linq;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000CE RID: 206
	[HandlerCategory("vvTrade"), HandlerName("Профит последней закрытой\nименованной позиции")]
	public class LastClosedNamedPositionProfit : IOneSourceHandler, IDoubleReturns, IValuesHandler, IHandler, ISecurityInputs
	{
		// Token: 0x060006F4 RID: 1780 RVA: 0x0001EFD0 File Offset: 0x0001D1D0
		public double Execute(ISecurity sec, int barNum)
		{
			IEnumerable<IPosition> closedForBar = sec.get_Positions().GetClosedForBar(barNum);
			int num = closedForBar.Count<IPosition>();
			if (num < 1)
			{
				return 0.0;
			}
			for (int i = num - 1; i >= 0; i--)
			{
				if (closedForBar.ElementAt(i).get_EntrySignalName() == this.Name)
				{
					return closedForBar.ElementAt(i).Profit();
				}
			}
			return 0.0;
		}

		// Token: 0x1700025A RID: 602
		[HandlerParameter(true, "", NotOptimized = true)]
		public string Name
		{
			// Token: 0x060006F2 RID: 1778 RVA: 0x0001EFBD File Offset: 0x0001D1BD
			get;
			// Token: 0x060006F3 RID: 1779 RVA: 0x0001EFC5 File Offset: 0x0001D1C5
			set;
		}
	}
}
