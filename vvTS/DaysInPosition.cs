using System;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000C8 RID: 200
	[HandlerCategory("vvTrade"), HandlerName("Дней в позиции")]
	public class DaysInPosition : IPosition2Double, IValuesHandler, IHandler, IOneSourceHandler, IDoubleReturns, IPositionInputs
	{
		// Token: 0x060006DD RID: 1757 RVA: 0x0001EC04 File Offset: 0x0001CE04
		public double Execute(IPosition pos, int barNum)
		{
			if (pos == null)
			{
				return 0.0;
			}
			DateTime date = pos.get_EntryBar().get_Date();
			DateTime date2 = pos.get_Security().get_Bars()[barNum].get_Date();
			return (date2 - date).TotalDays;
		}
	}
}
