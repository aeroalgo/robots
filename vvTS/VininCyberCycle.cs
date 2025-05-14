using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000142 RID: 322
	[HandlerCategory("vvIndicators"), HandlerName("VininI CyberCycle")]
	public class VininCyberCycle : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x060009F0 RID: 2544 RVA: 0x000291F6 File Offset: 0x000273F6
		public IList<double> Execute(IList<double> src)
		{
			return VininCyberCycle.GenVininCyberCycle(src, this.Alpha, this.Betta);
		}

		// Token: 0x060009EF RID: 2543 RVA: 0x00029034 File Offset: 0x00027234
		public static IList<double> GenVininCyberCycle(IList<double> src, double alpha, double betta)
		{
			int arg_06_0 = src.Count;
			if (alpha == 0.0)
			{
				alpha = 0.07;
			}
			if (betta == 0.0)
			{
				betta = 1.0;
			}
			IList<double> list = new double[src.Count];
			IList<double> list2 = new double[src.Count];
			IList<double> list3 = new double[src.Count];
			double num = 1.0 - 0.5 * alpha;
			double num2 = 1.0 - alpha;
			for (int i = 4; i < src.Count; i++)
			{
				list[i] = betta * (src[i] + 2.0 * src[i - 1] + 2.0 * src[i - 2] + src[i - 3]) / 6.0;
			}
			for (int j = 2; j < src.Count; j++)
			{
				list2[j] = num * num * (list[j] - 2.0 * list[j - 1] + list[j - 2]) - num2 * num2 * list2[j - 2] + 2.0 * num2 * list2[j - 1];
				list3[j] = 100.0 * (Math.Exp(2.0 * list2[j]) - 1.0) / (Math.Exp(2.0 * list2[j]) + 1.0);
			}
			return list3;
		}

		// Token: 0x17000340 RID: 832
		[HandlerParameter(true, "0.01", Min = "0.01", Max = "0.1", Step = "0.01")]
		public double Alpha
		{
			// Token: 0x060009EB RID: 2539 RVA: 0x00029011 File Offset: 0x00027211
			get;
			// Token: 0x060009EC RID: 2540 RVA: 0x00029019 File Offset: 0x00027219
			set;
		}

		// Token: 0x17000341 RID: 833
		[HandlerParameter(true, "0.01", Min = "0.01", Max = "0.1", Step = "0.01")]
		public double Betta
		{
			// Token: 0x060009ED RID: 2541 RVA: 0x00029022 File Offset: 0x00027222
			get;
			// Token: 0x060009EE RID: 2542 RVA: 0x0002902A File Offset: 0x0002722A
			set;
		}

		// Token: 0x17000342 RID: 834
		public IContext Context
		{
			// Token: 0x060009F1 RID: 2545 RVA: 0x0002920A File Offset: 0x0002740A
			get;
			// Token: 0x060009F2 RID: 2546 RVA: 0x00029212 File Offset: 0x00027412
			set;
		}
	}
}
