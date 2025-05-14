using System;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000101 RID: 257
	[HandlerCategory("vvTrade"), HandlerName("Минут в позиции")]
	public class MinutesInPosition : IPosition2Double, IValuesHandler, IHandler, IOneSourceHandler, IDoubleReturns, IPositionInputs
	{
		// Token: 0x06000768 RID: 1896 RVA: 0x000209E8 File Offset: 0x0001EBE8
		public double Execute(IPosition pos, int barNum)
		{
			if (pos == null)
			{
				return 0.0;
			}
			DateTime date = pos.get_EntryBar().get_Date();
			DateTime date2 = pos.get_Security().get_Bars()[barNum].get_Date();
			return (date2 - date).TotalMinutes;
		}
	}
}
