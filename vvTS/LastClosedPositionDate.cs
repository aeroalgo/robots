using System;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000D4 RID: 212
	[HandlerCategory("vvTrade"), HandlerName("Дата посл. закр. позиции")]
	public class LastClosedPositionDate : IOneSourceHandler, IDoubleReturns, IValuesHandler, IHandler, ISecurityInputs
	{
		// Token: 0x06000704 RID: 1796 RVA: 0x0001F728 File Offset: 0x0001D928
		public double Execute(ISecurity sec, int barNum)
		{
			IPosition position = sec.get_Positions().GetLastPositionClosed(barNum);
			if (position != null && position.get_ExitBarNum() > barNum)
			{
				position = null;
			}
			DateTime dateTime = (position == null) ? DateTime.MinValue : position.get_EntryBar().get_Date();
			return (double)(dateTime.Year % 100) * 10000.0 + (double)dateTime.Month * 100.0 + (double)dateTime.Day;
		}
	}
}
