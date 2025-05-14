using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000D0 RID: 208
	[HandlerCategory("vvTrade"), HandlerName("Таймфрейм")]
	public class GetTimeframe : IOneSourceHandler, IDoubleReturns, IValuesHandler, IHandler, ISecurityInputs
	{
		// Token: 0x060006F8 RID: 1784 RVA: 0x0001F088 File Offset: 0x0001D288
		public double Execute(ISecurity src, int barNum)
		{
			if (barNum < 3)
			{
				return 0.0;
			}
			int num = 0;
			IList<Bar> bars = src.get_Bars();
			if (bars[1].get_Date().Second != bars[0].get_Date().Second && bars[2].get_Date().Second != bars[1].get_Date().Second)
			{
				num = 1;
			}
			else if (bars[1].get_Date().Minute != bars[0].get_Date().Minute && bars[2].get_Date().Minute != bars[1].get_Date().Minute)
			{
				num = 2;
			}
			else if (bars[1].get_Date().Hour != bars[0].get_Date().Hour && bars[2].get_Date().Hour != bars[1].get_Date().Hour)
			{
				num = 3;
			}
			else if (bars[1].get_Date().Day != bars[0].get_Date().Day && bars[2].get_Date().Day != bars[1].get_Date().Day)
			{
				num = 4;
			}
			else if (bars[1].get_Date().Month != bars[0].get_Date().Month && bars[2].get_Date().Month != bars[1].get_Date().Month)
			{
				num = 5;
			}
			int value = 0;
			int num2 = 0;
			int num3 = 0;
			int num4 = 0;
			switch (num)
			{
			case 1:
				num2 = bars[1].get_Date().Second - bars[0].get_Date().Second;
				num3 = bars[2].get_Date().Second - bars[1].get_Date().Second;
				num4 = bars[3].get_Date().Second - bars[2].get_Date().Second;
				break;
			case 2:
				num2 = bars[1].get_Date().Minute - bars[0].get_Date().Minute;
				num3 = bars[2].get_Date().Minute - bars[1].get_Date().Minute;
				num4 = bars[3].get_Date().Minute - bars[2].get_Date().Minute;
				break;
			case 3:
				num2 = bars[1].get_Date().Hour - bars[0].get_Date().Hour;
				num3 = bars[2].get_Date().Hour - bars[1].get_Date().Hour;
				num4 = bars[3].get_Date().Hour - bars[2].get_Date().Hour;
				break;
			case 4:
				num2 = bars[1].get_Date().Day - bars[0].get_Date().Day;
				num3 = bars[2].get_Date().Day - bars[1].get_Date().Day;
				num4 = bars[3].get_Date().Day - bars[2].get_Date().Day;
				break;
			case 5:
				num2 = bars[1].get_Date().Month - bars[0].get_Date().Month;
				num3 = bars[2].get_Date().Month - bars[1].get_Date().Month;
				num4 = bars[3].get_Date().Month - bars[2].get_Date().Month;
				break;
			}
			if (num2 == num3 || num2 == num4)
			{
				value = num2;
			}
			else if (num4 == num3)
			{
				value = num3;
			}
			return Convert.ToDouble(value);
		}
	}
}
