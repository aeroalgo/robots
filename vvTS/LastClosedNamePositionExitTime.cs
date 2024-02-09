using System;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000D7 RID: 215
	[HandlerCategory("vvTrade"), HandlerName("Время выхода из посл. именован. позиции")]
	public class LastClosedNamePositionExitTime : IOneSourceHandler, IDoubleReturns, IValuesHandler, IHandler, ISecurityInputs
	{
		// Token: 0x0600070C RID: 1804 RVA: 0x0001F89C File Offset: 0x0001DA9C
		public double Execute(ISecurity sec, int barNum)
		{
			IPosition position = sec.get_Positions().GetLastForSignal(this.Name);
			if (position != null && (position.IsActiveForbar(barNum) || position.get_ExitBarNum() > barNum))
			{
				position = null;
			}
			DateTime dateTime = (position == null) ? DateTime.MinValue : position.get_ExitBar().get_Date();
			return (double)dateTime.Hour * 10000.0 + (double)dateTime.Minute * 100.0 + (double)dateTime.Second;
		}

		// Token: 0x1700025C RID: 604
		[HandlerParameter(true, "", NotOptimized = true)]
		public string Name
		{
			// Token: 0x0600070A RID: 1802 RVA: 0x0001F88B File Offset: 0x0001DA8B
			get;
			// Token: 0x0600070B RID: 1803 RVA: 0x0001F893 File Offset: 0x0001DA93
			set;
		}
	}
}
