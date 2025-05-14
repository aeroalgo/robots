using System;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000D5 RID: 213
	[HandlerCategory("vvTrade"), HandlerName("Время выхода из посл. позиции")]
	public class LastClosedPositionExitTime : IOneSourceHandler, IDoubleReturns, IValuesHandler, IHandler, ISecurityInputs
	{
		// Token: 0x06000706 RID: 1798 RVA: 0x0001F7A0 File Offset: 0x0001D9A0
		public double Execute(ISecurity sec, int barNum)
		{
			IPosition position = sec.get_Positions().GetLastPositionClosed(barNum);
			if (position != null && position.get_ExitBarNum() > barNum)
			{
				position = null;
			}
			DateTime dateTime = (position == null) ? DateTime.MinValue : position.get_ExitBar().get_Date();
			return (double)dateTime.Hour * 10000.0 + (double)dateTime.Minute * 100.0 + (double)dateTime.Second;
		}
	}
}
