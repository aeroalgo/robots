using System;
using System.Linq;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000CC RID: 204
	[HandlerCategory("vvTrade"), HandlerName("Профит N-й сделки (с конца)")]
	public class NLastDealProfit : IOneSourceHandler, IDoubleReturns, IValuesHandler, IHandler, ISecurityInputs
	{
		// Token: 0x060006ED RID: 1773 RVA: 0x0001EEE4 File Offset: 0x0001D0E4
		public double Execute(ISecurity sec, int barNum)
		{
			IPosition[] array = (from pos in sec.get_Positions()
			where !pos.IsActiveForbar(barNum) && pos.get_EntryBarNum() < barNum
			orderby pos.get_ExitBar().get_Date()
			select pos).ToArray<IPosition>();
			if (array.Length < 1)
			{
				return 0.0;
			}
			if (array.Length - this.N < 0 || this.N < 0)
			{
				return 0.0;
			}
			return array[array.Length - this.N].Profit();
		}

		// Token: 0x17000259 RID: 601
		[HandlerParameter(true, "", NotOptimized = true)]
		public int N
		{
			// Token: 0x060006EB RID: 1771 RVA: 0x0001EE9D File Offset: 0x0001D09D
			get;
			// Token: 0x060006EC RID: 1772 RVA: 0x0001EEA5 File Offset: 0x0001D0A5
			set;
		}
	}
}
