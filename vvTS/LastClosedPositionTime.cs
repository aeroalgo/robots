using System;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000D3 RID: 211
	[HandlerCategory("vvTrade"), HandlerName("Время посл. закр. позиции")]
	public class LastClosedPositionTime : IOneSourceHandler, IDoubleReturns, IValuesHandler, IHandler, ISecurityInputs
	{
		// Token: 0x06000702 RID: 1794 RVA: 0x0001F6B4 File Offset: 0x0001D8B4
		public double Execute(ISecurity sec, int barNum)
		{
			IPosition position = sec.get_Positions().GetLastPositionClosed(barNum);
			if (position != null && position.get_ExitBarNum() > barNum)
			{
				position = null;
			}
			DateTime dateTime = (position == null) ? DateTime.MinValue : position.get_EntryBar().get_Date();
			return (double)dateTime.Hour * 10000.0 + (double)dateTime.Minute * 100.0 + (double)dateTime.Second;
		}
	}
}
