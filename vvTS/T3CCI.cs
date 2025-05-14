using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200001C RID: 28
	[HandlerCategory("vvIndicators"), HandlerName("T3 CCI")]
	public class T3CCI : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x060000F0 RID: 240 RVA: 0x000050D6 File Offset: 0x000032D6
		public IList<double> Execute(ISecurity src)
		{
			return T3CCI.GenT3CCI(src, this.CCI_Period, this.T3_Period, this.T3_Hot);
		}

		// Token: 0x060000EF RID: 239 RVA: 0x00004EC0 File Offset: 0x000030C0
		public static IList<double> GenT3CCI(ISecurity src, int _CCI_Period, int _T3_Period, double _T3_Hot)
		{
			int count = src.get_Bars().Count;
			double num = 0.0;
			double num2 = 0.0;
			double num3 = 0.0;
			double num4 = 0.0;
			double num5 = 0.0;
			double num6 = 0.0;
			double[] array = new double[count];
			double num7 = _T3_Hot * _T3_Hot;
			double num8 = num7 * _T3_Hot;
			double num9 = -num8;
			double num10 = 3.0 * (num7 + num8);
			double num11 = -3.0 * (2.0 * num7 + _T3_Hot + num8);
			double num12 = 1.0 + 3.0 * _T3_Hot + num8 + 3.0 * num7;
			double num13 = (double)_T3_Period;
			if (num13 < 1.0)
			{
				num13 = 1.0;
			}
			num13 = 1.0 + 0.5 * (num13 - 1.0);
			double num14 = 2.0 / (num13 + 1.0);
			double num15 = 1.0 - num14;
			IList<double> list = CCI.GenCCI(src.get_Bars(), _CCI_Period, 0, 100);
			for (int i = _CCI_Period; i < count; i++)
			{
				num = num14 * list[i] + num15 * num;
				num2 = num14 * num + num15 * num2;
				num3 = num14 * num2 + num15 * num3;
				num4 = num14 * num3 + num15 * num4;
				num5 = num14 * num4 + num15 * num5;
				num6 = num14 * num5 + num15 * num6;
				array[i] = num9 * num6 + num10 * num5 + num11 * num4 + num12 * num3;
			}
			return array;
		}

		// Token: 0x1700004B RID: 75
		[HandlerParameter(true, "14", Min = "3", Max = "20", Step = "1")]
		public int CCI_Period
		{
			// Token: 0x060000E9 RID: 233 RVA: 0x00004E8B File Offset: 0x0000308B
			get;
			// Token: 0x060000EA RID: 234 RVA: 0x00004E93 File Offset: 0x00003093
			set;
		}

		// Token: 0x1700004E RID: 78
		public IContext Context
		{
			// Token: 0x060000F1 RID: 241 RVA: 0x000050F0 File Offset: 0x000032F0
			get;
			// Token: 0x060000F2 RID: 242 RVA: 0x000050F8 File Offset: 0x000032F8
			set;
		}

		// Token: 0x1700004D RID: 77
		[HandlerParameter(true, "0.618", Min = "0", Max = "1", Step = "0.01")]
		public double T3_Hot
		{
			// Token: 0x060000ED RID: 237 RVA: 0x00004EAD File Offset: 0x000030AD
			get;
			// Token: 0x060000EE RID: 238 RVA: 0x00004EB5 File Offset: 0x000030B5
			set;
		}

		// Token: 0x1700004C RID: 76
		[HandlerParameter(true, "3", Min = "5", Max = "20", Step = "1")]
		public int T3_Period
		{
			// Token: 0x060000EB RID: 235 RVA: 0x00004E9C File Offset: 0x0000309C
			get;
			// Token: 0x060000EC RID: 236 RVA: 0x00004EA4 File Offset: 0x000030A4
			set;
		}
	}
}
