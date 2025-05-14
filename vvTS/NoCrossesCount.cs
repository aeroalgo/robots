using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000BA RID: 186
	[HandlerCategory("vvTrade"), HandlerName("Сколько баров не было пересечения"), InputInfo(1, "Список чисел 2"), InputInfo(0, "Список чисел 1")]
	public class NoCrossesCount : IDoubleAccumHandler, ITwoSourcesHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x060006A5 RID: 1701 RVA: 0x0001E3F0 File Offset: 0x0001C5F0
		public IList<double> Execute(IList<double> src, IList<double> src1)
		{
			return this.Context.GetData("NoCrossesCount", new string[]
			{
				src.GetHashCode().ToString(),
				src1.GetHashCode().ToString()
			}, () => NoCrossesCount.GenNoCrossesCount(src, src1));
		}

		// Token: 0x060006A4 RID: 1700 RVA: 0x0001E2F0 File Offset: 0x0001C4F0
		public static IList<double> GenNoCrossesCount(IList<double> src, IList<double> src1)
		{
			IList<double> list = new double[src1.Count];
			int num = 0;
			for (int i = 0; i < src1.Count; i++)
			{
				if (i == 0)
				{
					if (src[i] > src1[i])
					{
						num = 1;
					}
					if (src[i] < src1[i])
					{
						num = -1;
					}
					list[i] = 0.0;
				}
				else if ((src[i] > src1[i] && num == 1) || (src[i] < src1[i] && num == -1))
				{
					list[i] = list[i - 1] + 1.0;
				}
				else
				{
					if (src[i] > src1[i])
					{
						num = 1;
					}
					if (src[i] < src1[i])
					{
						num = -1;
					}
					list[i] = 0.0;
				}
			}
			return list;
		}

		// Token: 0x1700024C RID: 588
		public IContext Context
		{
			// Token: 0x060006A6 RID: 1702 RVA: 0x0001E468 File Offset: 0x0001C668
			get;
			// Token: 0x060006A7 RID: 1703 RVA: 0x0001E470 File Offset: 0x0001C670
			set;
		}
	}
}
