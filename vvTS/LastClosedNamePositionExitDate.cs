using System;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000D8 RID: 216
	[HandlerCategory("vvTrade"), HandlerName("Дата выхода из последней\nименованной позиции")]
	public class LastClosedNamePositionExitDate : IOneSourceHandler, IDoubleReturns, IValuesHandler, IHandler, ISecurityInputs
	{
		// Token: 0x06000710 RID: 1808 RVA: 0x0001F930 File Offset: 0x0001DB30
		public double Execute(ISecurity sec, int barNum)
		{
			IPosition position = sec.get_Positions().GetLastForSignal(this.Name);
			if (position != null && (position.IsActiveForbar(barNum) || position.get_ExitBarNum() > barNum))
			{
				position = null;
			}
			DateTime dateTime = (position == null) ? DateTime.MinValue : position.get_ExitBar().get_Date();
			return (double)(dateTime.Year % 100) * 10000.0 + (double)dateTime.Month * 100.0 + (double)dateTime.Day;
		}

		// Token: 0x1700025D RID: 605
		[HandlerParameter(true, "", NotOptimized = true)]
		public string Name
		{
			// Token: 0x0600070E RID: 1806 RVA: 0x0001F91E File Offset: 0x0001DB1E
			get;
			// Token: 0x0600070F RID: 1807 RVA: 0x0001F926 File Offset: 0x0001DB26
			set;
		}
	}
}
