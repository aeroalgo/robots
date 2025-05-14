using System;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000D6 RID: 214
	[HandlerCategory("vvTrade"), HandlerName("Дата выхода из посл. позиции")]
	public class LastClosedPositionExitDate : IOneSourceHandler, IDoubleReturns, IValuesHandler, IHandler, ISecurityInputs
	{
		// Token: 0x06000708 RID: 1800 RVA: 0x0001F814 File Offset: 0x0001DA14
		public double Execute(ISecurity sec, int barNum)
		{
			IPosition position = sec.get_Positions().GetLastPositionClosed(barNum);
			if (position != null && position.get_ExitBarNum() > barNum)
			{
				position = null;
			}
			DateTime dateTime = (position == null) ? DateTime.MinValue : position.get_ExitBar().get_Date();
			return (double)(dateTime.Year % 100) * 10000.0 + (double)dateTime.Month * 100.0 + (double)dateTime.Day;
		}
	}
}
