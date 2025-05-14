using System;
using System.Linq;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000CB RID: 203
	[HandlerCategory("vvTrade"), HandlerName("Всего прибыльных сделок")]
	public class ProfitDealsCount : IOneSourceHandler, IDoubleReturns, IValuesHandler, IHandler, ISecurityInputs
	{
		// Token: 0x060006E6 RID: 1766 RVA: 0x0001EDF8 File Offset: 0x0001CFF8
		public double Execute(ISecurity sec, int barNum)
		{
			return (double)(from pos in sec.get_Positions()
			orderby pos.get_ExitBarNum() descending
			where !pos.IsActiveForbar(barNum)
			select pos.Profit()).TakeWhile((double profit) => profit >= 0.0).Count<double>();
		}
	}
}
