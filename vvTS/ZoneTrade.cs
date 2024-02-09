using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200007A RID: 122
	[HandlerCategory("vvWilliams"), HandlerName("Williams ZoneTrade")]
	public class ZoneTrade : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x06000461 RID: 1121 RVA: 0x000170B0 File Offset: 0x000152B0
		public IList<double> Execute(ISecurity sec)
		{
			IList<double> closePrices = sec.get_ClosePrices();
			double item = 0.0;
			int num = 0;
			int num2 = 0;
			IList<double> list = new List<double>(closePrices.Count);
			IList<double> list2 = WilliamsAO.GenWilliamsAO(sec, this.Context, this.Period1, this.Period2);
			IList<double> list3 = WilliamsAC.GenWilliamsAC(sec, this.Context, this.Period1, this.Period2, 5);
			for (int i = 0; i < closePrices.Count; i++)
			{
				if (i < 1)
				{
					item = 0.0;
				}
				else
				{
					double num3 = list3[i];
					double num4 = list3[i - 1];
					if (num3 > num4)
					{
						num = 1;
					}
					if (num3 < num4)
					{
						num = 2;
					}
					double num5 = list2[i];
					double num6 = list2[i - 1];
					if (num5 > num6)
					{
						num2 = 1;
					}
					if (num5 < num6)
					{
						num2 = 2;
					}
					if (num == 1 && num2 == 1)
					{
						item = 1.0;
					}
					if (num == 2 && num2 == 2)
					{
						item = -1.0;
					}
					if ((num == 1 && num2 == 2) || (num == 2 && num2 == 1))
					{
						item = 0.0;
					}
				}
				list.Add(item);
			}
			if (this.Chart == 1)
			{
				IPane pane = this.Context.CreatePane("ZoneTrade", 30.0, true);
				IGraphList graphList = pane.AddList(sec.get_Symbol(), sec, 0, 41120, 0);
				for (int j = 0; j < sec.get_Bars().Count; j++)
				{
					int num7 = 9145227;
					if (list[j] > 0.0)
					{
						num7 = 100119;
					}
					if (list[j] < 0.0)
					{
						num7 = 16717848;
					}
					graphList.SetColor(j, num7);
				}
			}
			return list;
		}

		// Token: 0x1700017C RID: 380
		[HandlerParameter(true, "1", NotOptimized = true)]
		public int Chart
		{
			// Token: 0x0600045B RID: 1115 RVA: 0x0001707D File Offset: 0x0001527D
			get;
			// Token: 0x0600045C RID: 1116 RVA: 0x00017085 File Offset: 0x00015285
			set;
		}

		// Token: 0x1700017F RID: 383
		public IContext Context
		{
			// Token: 0x06000462 RID: 1122 RVA: 0x00017298 File Offset: 0x00015498
			get;
			// Token: 0x06000463 RID: 1123 RVA: 0x000172A0 File Offset: 0x000154A0
			set;
		}

		// Token: 0x1700017D RID: 381
		[HandlerParameter(true, "5", Min = "5", Max = "100", Step = "1")]
		public int Period1
		{
			// Token: 0x0600045D RID: 1117 RVA: 0x0001708E File Offset: 0x0001528E
			get;
			// Token: 0x0600045E RID: 1118 RVA: 0x00017096 File Offset: 0x00015296
			set;
		}

		// Token: 0x1700017E RID: 382
		[HandlerParameter(true, "34", Min = "5", Max = "100", Step = "1")]
		public int Period2
		{
			// Token: 0x0600045F RID: 1119 RVA: 0x0001709F File Offset: 0x0001529F
			get;
			// Token: 0x06000460 RID: 1120 RVA: 0x000170A7 File Offset: 0x000152A7
			set;
		}
	}
}
