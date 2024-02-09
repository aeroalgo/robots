using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000021 RID: 33
	[HandlerCategory("vvIndicators"), HandlerName("FisherCyberCycle")]
	public class FisherCyberCycle : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x0600012A RID: 298 RVA: 0x00005DC3 File Offset: 0x00003FC3
		public IList<double> Execute(IList<double> src)
		{
			return FisherCyberCycle.GenFisherCyberCycle(src, this.Period, this.TriggerLine);
		}

		// Token: 0x06000129 RID: 297 RVA: 0x00005B10 File Offset: 0x00003D10
		public static IList<double> GenFisherCyberCycle(IList<double> src, int _period, bool _TriggerLine)
		{
			int count = src.Count;
			if (_period < 4)
			{
				return null;
			}
			double num = 0.07;
			List<double> list = new List<double>(count);
			List<double> list2 = new List<double>(count);
			double[] array = new double[count];
			double[] array2 = new double[count];
			double[] array3 = new double[count];
			for (int i = 0; i < _period; i++)
			{
				list.Add(0.0);
				list2.Add(0.0);
				array[i] = 0.0;
				array2[i] = 0.0;
				array3[i] = 0.0;
			}
			for (int j = _period; j < count; j++)
			{
				array[j] = (src[j] + 2.0 * src[j - 1] + 2.0 * src[j - 2] + src[j - 3]) / 6.0;
				array2[j] = (1.0 - 0.5 * num) * (1.0 - 0.5 * num) * (array[j] - 2.0 * array[j - 1] + array[j - 2]) + 2.0 * (1.0 - num) * array2[j - 1] - (1.0 - num) * (1.0 - num) * array2[j - 2];
				double num2 = array2[j];
				double num3 = array2[j];
				for (int k = 0; k < _period; k++)
				{
					double val = array2[j - k];
					num2 = Math.Max(num2, val);
					num3 = Math.Min(num3, val);
				}
				array3[j] = 0.0;
				if (num2 != num3)
				{
					array3[j] = (array2[j] - num3) / (num2 - num3);
				}
				double num4 = (4.0 * array3[j] + 3.0 * array3[j - 1] + 2.0 * array3[j - 2] + array3[j - 3]) / 10.0;
				num4 = 0.5 * Math.Log((1.0 + 1.98 * (num4 - 0.5)) / (1.0 - 1.98 * (num4 - 0.5)));
				list.Add(num4);
				list2.Add(list[j - 1]);
			}
			if (!_TriggerLine)
			{
				return list;
			}
			return list2;
		}

		// Token: 0x17000063 RID: 99
		public IContext Context
		{
			// Token: 0x0600012B RID: 299 RVA: 0x00005DD7 File Offset: 0x00003FD7
			get;
			// Token: 0x0600012C RID: 300 RVA: 0x00005DDF File Offset: 0x00003FDF
			set;
		}

		// Token: 0x17000061 RID: 97
		[HandlerParameter(true, "8", Min = "1", Max = "50", Step = "1")]
		public int Period
		{
			// Token: 0x06000125 RID: 293 RVA: 0x00005AEE File Offset: 0x00003CEE
			get;
			// Token: 0x06000126 RID: 294 RVA: 0x00005AF6 File Offset: 0x00003CF6
			set;
		}

		// Token: 0x17000062 RID: 98
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool TriggerLine
		{
			// Token: 0x06000127 RID: 295 RVA: 0x00005AFF File Offset: 0x00003CFF
			get;
			// Token: 0x06000128 RID: 296 RVA: 0x00005B07 File Offset: 0x00003D07
			set;
		}
	}
}
